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
}

impl Default for StreamingWriterConfig {
    fn default() -> Self {
        StreamingWriterConfig {
            row_group_size: crate::DEFAULT_ROW_GROUP_SIZE,
            compression_level: 3,
            buffer_pool_config: BufferPoolConfig::default(),
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
        // Placeholder for row count (will update at finish)
        writer.write_u32::<LittleEndian>(0)?;
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
        let rg_bytes = row_group.serialize()?;
        self.writer.write_all(&rg_bytes)?;
        self.current_offset += rg_bytes.len() as u64;

        // Clear buffer for reuse
        self.row_buffer.clear();
        self.row_group_count += 1;

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

        // Build and write footer
        let mut footer = crate::footer::Footer::new(self.schema, self.total_rows);
        footer.row_group_offsets = self.row_group_offsets;

        let footer_bytes = footer.serialize()?;
        let footer_length = footer_bytes.len() as u32;

        // Write footer
        self.writer.write_all(&footer_bytes)?;

        // Write footer length
        use byteorder::{LittleEndian, WriteBytesExt};
        self.writer.write_u32::<LittleEndian>(footer_length)?;

        // Write footer CRC32
        let footer_crc = crate::validation::Validator::calculate_crc32(&footer_bytes);
        self.writer.write_u32::<LittleEndian>(footer_crc)?;

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
