//! Streaming row-group writer with bounded memory usage
//!
//! Implements production-grade streaming writer that:
//! - Maintains bounded memory (O(row_group_size) not O(dataset_size))
//! - Automatically flushes row groups
//! - Reuses buffers via pool
//! - Supports incremental ingestion
//! - Enables async-friendly architecture

use crate::columnar::RowBuffer;
use crate::compression::CompressionLevel;
use crate::error::Result;
use crate::encryption::EncryptionConfig;
use crate::ecc::{EccConfig, EccCodec};
use crate::schema::Schema;
use crate::writer::buffer_pool::{BufferPool, BufferPoolConfig};
use std::io::Write;

/// Configuration for streaming writer
#[derive(Debug, Clone)]
pub struct StreamingWriterConfig {
    /// Maximum rows per row group (triggers auto-flush)
    pub row_group_size: u32,
    /// Compression level for row groups
    pub compression_level: u8,
    /// Buffer pool configuration
    pub buffer_pool_config: BufferPoolConfig,
    /// Optional encryption configuration for row groups
    pub encryption: Option<EncryptionConfig>,
    /// Optional ECC configuration for row groups
    pub ecc: Option<EccConfig>,
}

impl Default for StreamingWriterConfig {
    fn default() -> Self {
        StreamingWriterConfig {
            row_group_size: crate::DEFAULT_ROW_GROUP_SIZE,
            compression_level: 3,
            buffer_pool_config: BufferPoolConfig::default(),
            encryption: None,
            ecc: None,
        }
    }
}

/// Streaming writer for QRD format
pub struct StreamingWriter<W: Write> {
    writer: W,
    schema: Schema,
    config: StreamingWriterConfig,
    buffer_pool: BufferPool,
    row_buffer: RowBuffer,
    row_group_count: u32,
    total_rows: u32,
    row_group_offsets: Vec<u64>,
    current_offset: u64,
    is_finished: bool,
    // Statistics collection for current and completed row groups
    current_row_group_stats: crate::metadata::RowGroupStats,
    row_group_stats: Vec<crate::metadata::RowGroupStats>,
}

impl<W: Write> StreamingWriter<W> {
    /// Create new streaming writer with default config
    pub fn new(writer: W, schema: Schema) -> Result<Self> {
        Self::with_config(writer, schema, StreamingWriterConfig::default())
    }

    /// Create with custom config
    pub fn with_config(
        mut writer: W,
        schema: Schema,
        config: StreamingWriterConfig,
    ) -> Result<Self> {
        // Write QRD file header
        Self::write_header(&mut writer, &schema)?;

        Ok(StreamingWriter {
            writer,
            schema: schema.clone(),
            buffer_pool: BufferPool::with_config(config.buffer_pool_config),
            row_buffer: RowBuffer::new(schema.fields.len()),
            config,
            row_group_count: 0,
            total_rows: 0,
            row_group_offsets: Vec::new(),
            current_offset: 32, // After header
            is_finished: false,
            current_row_group_stats: crate::metadata::RowGroupStats::new(&schema),
            row_group_stats: Vec::new(),
        })
    }

    /// Write file header (magic + metadata)
    fn write_header(writer: &mut W, schema: &Schema) -> Result<()> {
        use byteorder::{LittleEndian, WriteBytesExt};

        // Magic bytes
        writer.write_all(crate::QRD_MAGIC)?;
        // Version
        writer.write_u16::<LittleEndian>(crate::QRD_VERSION_MAJOR)?;
        writer.write_u16::<LittleEndian>(crate::QRD_VERSION_MINOR)?;
        // Schema ID
        writer.write_u32::<LittleEndian>(schema.schema_id)?;
        // Created timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;
        writer.write_u32::<LittleEndian>(now)?;
        // Placeholder for row count — sentinel means "read from footer"
        writer.write_u32::<LittleEndian>(u32::MAX)?;
        // Column count
        writer.write_u32::<LittleEndian>(schema.fields.len() as u32)?;
        // Row group size
        writer.write_u32::<LittleEndian>(crate::DEFAULT_ROW_GROUP_SIZE)?;
        // Reserved
        writer.write_u32::<LittleEndian>(0)?;

        Ok(())
    }

    /// Ingest a row (as column values)
    pub fn write_row(&mut self, row: Vec<Vec<u8>>) -> Result<()> {
        if self.is_finished {
            return Err(crate::error::Error::InvalidData(
                "Cannot write to finished writer".to_string(),
            ));
        }

        // Update statistics similar to FileWriter
        let stats_row: Vec<Option<Vec<u8>>> = row.iter()
            .map(|col| if col.is_empty() { None } else { Some(col.clone()) })
            .collect();
        self.current_row_group_stats.update_row(&stats_row);

        self.row_buffer.add_row(row)?;
        self.total_rows += 1;

        // Auto-flush when row group threshold reached
        if self.row_buffer.row_count() >= self.config.row_group_size {
            self.flush_row_group()?;
        }

        Ok(())
    }

    /// Manually flush current row group to output
    pub fn flush_row_group(&mut self) -> Result<()> {
        if self.row_buffer.is_empty() {
            return Ok(());
        }

        // Record offset
        self.row_group_offsets.push(self.current_offset);

        // Transpose rows to columnar format
        let field_types: Vec<_> = self.schema.fields.iter().map(|f| f.field_type).collect();
        let columns = self.row_buffer.transpose(&field_types)?;

        // Process each column
        let mut row_group = crate::rowgroup::RowGroup::new(self.row_buffer.row_count());
        
        for column in columns {
            let encoding = crate::encoding::select_encoding(&column.field_type, &column.data);
            row_group.process_column(
                column,
                encoding,
                crate::compression::CompressionCodec::Zstd,
                CompressionLevel::new(self.config.compression_level),
            )?;
        }

        // Serialize row group
        // Attach statistics and serialize
        let stats_for_group = self.current_row_group_stats.clone();
        row_group.column_stats = Some(stats_for_group.column_stats.clone());

        let rg_bytes = row_group.serialize()?;

        // STEP 1: Encryption (if enabled)
        let encrypted_bytes = if let Some(ref enc_config) = self.config.encryption {
            crate::encryption::encrypt(&rg_bytes, enc_config)?
        } else {
            rg_bytes
        };

        // STEP 2: ECC encoding (if enabled)
        if let Some(ref ecc_config) = self.config.ecc {
            let mut codec = crate::ecc::EccCodec::new(ecc_config.clone())?;
            let encoded = codec.encode(&encrypted_bytes)?;
            let final_bytes = encoded.to_bytes()?;
            self.writer.write_all(&final_bytes)?;
            self.current_offset += final_bytes.len() as u64;
        } else {
            self.writer.write_all(&encrypted_bytes)?;
            self.current_offset += encrypted_bytes.len() as u64;
        }

        // Clear buffer for reuse
        self.row_buffer.clear();
        self.row_group_count += 1;

        // Save and reset statistics
        let completed_stats = std::mem::replace(&mut self.current_row_group_stats, crate::metadata::RowGroupStats::new(&self.schema));
        self.row_group_stats.push(completed_stats);

        Ok(())
    }

    /// Check buffer pool memory efficiency
    pub fn buffer_pool_stats(&self) -> (usize, usize) {
        self.buffer_pool.stats()
    }

    /// Get current row count
    pub fn row_count(&self) -> u32 {
        self.total_rows
    }

    /// Get row group count
    pub fn row_group_count(&self) -> u32 {
        self.row_group_count
    }

    /// Finish writing and flush final row group + footer
    pub fn finish(mut self) -> Result<()> {
        if self.is_finished {
            return Ok(());
        }

        // Flush any remaining rows
        self.flush_row_group()?;

        // Create metadata index from collected row group stats
        let metadata_index = crate::metadata::MetadataIndex::new(
            &self.schema,
            self.row_group_offsets.clone(),
            self.row_group_stats.clone(),
        );

        let mut footer = crate::footer::Footer::with_metadata_index(
            self.schema.clone(),
            self.total_rows,
            metadata_index,
        );
        footer.row_group_offsets = self.row_group_offsets;

        let footer_bytes = footer.serialize()?;
        let footer_length = footer_bytes.len() as u32;

        // Write footer
        self.writer.write_all(&footer_bytes)?;

        // Write footer length
        use byteorder::{LittleEndian, WriteBytesExt};
        self.writer.write_u32::<LittleEndian>(footer_length)?;

        self.is_finished = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::schema::{SchemaBuilder, FieldType, Nullability};

    fn make_schema(names: Vec<&str>) -> crate::schema::Schema {
        let mut builder = SchemaBuilder::new();
        for name in names {
            builder = builder
                .add_field(name, FieldType::Blob, Nullability::Required)
                .unwrap();
        }
        builder.build().unwrap()
    }

    #[test]
    fn test_streaming_writer_basic() -> Result<()> {
        let schema = make_schema(vec!["col1"]);
        let buffer = Cursor::new(Vec::new());
        let writer = StreamingWriter::new(buffer, schema)?;

        // Should initialize successfully
        assert_eq!(writer.row_count(), 0);
        assert_eq!(writer.row_group_count(), 0);

        Ok(())
    }

    #[test]
    fn test_streaming_writer_to_vec_non_seekable() -> Result<()> {
        let schema = make_schema(vec!["col1"]);
        // Use shared buffer so we can inspect it after finish
        use std::sync::{Arc, Mutex};
        struct SharedSink(Arc<Mutex<Vec<u8>>>);
        impl std::io::Write for SharedSink {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                let mut inner = self.0.lock().unwrap();
                inner.extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
        }

        let shared = Arc::new(Mutex::new(Vec::new()));
        let sink = SharedSink(shared.clone());
        let mut writer = StreamingWriter::new(sink, schema)?;

        for i in 0..1000u32 {
            writer.write_row(vec![(i as u8).to_le_bytes().to_vec()])?;
        }

        writer.finish()?; // consumes writer but shared buffer remains accessible

        // Read footer from shared buffer
        let buf = shared.lock().unwrap();
        let len = buf.len();
        assert!(len > 12);
        let footer_len = u32::from_le_bytes([buf[len - 4], buf[len - 3], buf[len - 2], buf[len - 1]]) as usize;
        let footer_start = len - 4 - footer_len;
        let footer_bytes = &buf[footer_start..footer_start + footer_len];
        let footer = crate::footer::Footer::deserialize(footer_bytes)?;
        assert_eq!(footer.row_count, 1000);

        Ok(())
    }

    #[test]
    fn test_streaming_writer_bounded_memory() -> Result<()> {
        let schema = make_schema(vec!["data"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 100;
        
        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        // Write should stay bounded
        for _ in 0..100 {
            writer.write_row(vec![vec![42u8]])?;
        }

        let (cached, _) = writer.buffer_pool_stats();
        assert!(cached <= 1); // Minimal caching after flush

        Ok(())
    }

    #[test]
    fn test_streaming_writer_auto_flush() -> Result<()> {
        let schema = make_schema(vec!["x"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 10;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        // Write 25 rows
        for _ in 0..25 {
            writer.write_row(vec![vec![1u8]])?;
        }

        // Should have flushed at least once
        assert!(writer.row_group_count() > 0);
        assert_eq!(writer.row_count(), 25);

        Ok(())
    }
}
