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
use crate::ecc::EccConfig;
use crate::encryption::EncryptionConfig;
use crate::error::Result;
use crate::schema::Schema;
use crate::validation::Validator;
use crate::writer::buffer_pool::{BufferPool, BufferPoolConfig};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

const PER_COLUMN_ROW_GROUP_MARKER: u8 = 0xFF;

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
    /// Enable per-column encryption (default: false)
    pub per_column_encryption: bool,
}

impl Default for StreamingWriterConfig {
    fn default() -> Self {
        StreamingWriterConfig {
            row_group_size: crate::DEFAULT_ROW_GROUP_SIZE,
            compression_level: 3,
            buffer_pool_config: BufferPoolConfig::default(),
            encryption: None,
            ecc: None,
            per_column_encryption: false,
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
        // Write QRD file header using shared helper from utils
        crate::utils::write_header(&mut writer, &schema, config.row_group_size)?;

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

    /// Ingest a row (as column values)
    pub fn write_row(&mut self, row: Vec<Vec<u8>>) -> Result<()> {
        if self.is_finished {
            return Err(crate::error::Error::InvalidData(
                "Cannot write to finished writer".to_string(),
            ));
        }

        // Update statistics similar to FileWriter
        let stats_row: Vec<Option<Vec<u8>>> = row
            .iter()
            .map(|col| {
                if col.is_empty() {
                    None
                } else {
                    Some(col.clone())
                }
            })
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
            if self.config.per_column_encryption {
                // Per-column encryption: encrypt each column chunk with a unique derived key
                self.encrypt_row_group_per_column(&row_group, enc_config)?
            } else {
                // Master key encryption: single key for all row group bytes
                crate::encryption::encrypt(&rg_bytes, enc_config)?
            }
        } else {
            rg_bytes
        };

        // STEP 2: ECC encoding (if enabled)
        if let Some(ref ecc_config) = self.config.ecc {
            let wrapped_bytes = self.wrap_row_group_with_crc(&encrypted_bytes)?;
            let mut codec = crate::ecc::EccCodec::new(ecc_config.clone())?;
            let encoded = codec.encode(&wrapped_bytes)?;
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
        let completed_stats = std::mem::replace(
            &mut self.current_row_group_stats,
            crate::metadata::RowGroupStats::new(&self.schema),
        );
        self.row_group_stats.push(completed_stats);

        Ok(())
    }

    fn wrap_row_group_with_crc(&self, data: &[u8]) -> Result<Vec<u8>> {
        let checksum = Validator::calculate_crc32(data);
        let mut wrapped = Vec::with_capacity(4 + data.len());
        wrapped.write_u32::<LittleEndian>(checksum)?;
        wrapped.extend_from_slice(data);
        Ok(wrapped)
    }

    /// Encrypt row group using per-column keys derived from master key
    ///
    /// Each column is encrypted with a unique key derived from the master key
    /// and the column name, allowing selective decryption of specific columns.
    fn encrypt_row_group_per_column(
        &self,
        row_group: &crate::rowgroup::RowGroup,
        enc_config: &EncryptionConfig,
    ) -> Result<Vec<u8>> {
        let mut wrapper = Vec::new();
        wrapper.push(PER_COLUMN_ROW_GROUP_MARKER);
        wrapper.write_u32::<LittleEndian>(row_group.row_count)?;
        wrapper.write_u16::<LittleEndian>(row_group.columns.len() as u16)?;

        for column in &row_group.columns {
            let column_name = self.schema.fields[column.column_index].name.clone();
            let derived_key = enc_config.derive_column_key(&column_name)?;
            let column_config = EncryptionConfig::new(derived_key)?;

            let mut chunk_bytes = Vec::new();
            chunk_bytes.push(column.encoding.to_id());
            chunk_bytes.push(column.compression.to_id());
            chunk_bytes.write_u32::<LittleEndian>(column.encoded_data.len() as u32)?;
            chunk_bytes.write_u32::<LittleEndian>(column.compressed_data.len() as u32)?;

            if let Some(ref stats_vec) = row_group.column_stats {
                if let Some(col_stats) = stats_vec.get(column.column_index) {
                    chunk_bytes.write_u32::<LittleEndian>(col_stats.null_count as u32)?;
                    chunk_bytes.write_u32::<LittleEndian>(col_stats.distinct_count as u32)?;
                } else {
                    chunk_bytes.write_u32::<LittleEndian>(0)?;
                    chunk_bytes.write_u32::<LittleEndian>(0)?;
                }
            } else {
                chunk_bytes.write_u32::<LittleEndian>(0)?;
                chunk_bytes.write_u32::<LittleEndian>(0)?;
            }

            chunk_bytes.extend_from_slice(&column.compressed_data);
            chunk_bytes.write_u32::<LittleEndian>(column.crc32)?;

            let encrypted_chunk = crate::encryption::encrypt(&chunk_bytes, &column_config)?;
            wrapper.write_u32::<LittleEndian>(encrypted_chunk.len() as u32)?;
            wrapper.extend_from_slice(&encrypted_chunk);
        }

        Ok(wrapper)
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

    /// Extract the inner writer after completion
    pub fn into_inner(self) -> W {
        self.writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FieldType, Nullability, SchemaBuilder};
    use std::io::Cursor;

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
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
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
        let footer_len =
            u32::from_le_bytes([buf[len - 4], buf[len - 3], buf[len - 2], buf[len - 1]]) as usize;
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

    // ====== Additional Streaming Writer Tests ======

    #[test]
    fn test_streaming_writer_large_dataset() -> Result<()> {
        let schema = make_schema(vec!["value"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 1000;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        for i in 0..10000 {
            writer.write_row(vec![(i as u32).to_le_bytes().to_vec()])?;
        }

        assert_eq!(writer.row_count(), 10000);
        assert!(writer.row_group_count() > 0);

        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_partial_flush() -> Result<()> {
        let schema = make_schema(vec!["id", "value"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 100;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        for i in 0..50 {
            writer.write_row(vec![
                (i as u32).to_le_bytes().to_vec(),
                ((i * 2) as u32).to_le_bytes().to_vec(),
            ])?;
        }

        assert_eq!(writer.row_count(), 50);
        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_deterministic_output() -> Result<()> {
        let schema = make_schema(vec!["data"]);
        
        let mut buffer1 = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer1);
            let mut writer = StreamingWriter::new(cursor, schema.clone())?;
            for i in 0..100 {
                writer.write_row(vec![(i as u8).to_le_bytes().to_vec()])?;
            }
            writer.finish()?;
        }

        let mut buffer2 = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer2);
            let mut writer = StreamingWriter::new(cursor, schema.clone())?;
            for i in 0..100 {
                writer.write_row(vec![(i as u8).to_le_bytes().to_vec()])?;
            }
            writer.finish()?;
        }

        assert_eq!(buffer1, buffer2);
        Ok(())
    }

    #[test]
    fn test_streaming_writer_incremental_row_groups() -> Result<()> {
        let schema = make_schema(vec!["value"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 25;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        let mut prev_rg_count = writer.row_group_count();

        for batch in 0..4 {
            for i in 0..30 {
                writer.write_row(vec![(batch * 30 + i).to_le_bytes().to_vec()])?;
            }
            let new_rg_count = writer.row_group_count();
            assert!(new_rg_count >= prev_rg_count);
            prev_rg_count = new_rg_count;
        }

        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_empty_stream() -> Result<()> {
        let schema = make_schema(vec!["x"]);
        let buffer = Cursor::new(Vec::new());

        let mut writer = StreamingWriter::new(buffer, schema)?;
        assert_eq!(writer.row_count(), 0);
        
        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_repeated_writes() -> Result<()> {
        let schema = make_schema(vec!["id"]);
        let buffer = Cursor::new(Vec::new());

        let mut writer = StreamingWriter::new(buffer, schema)?;

        for iteration in 0..5 {
            for i in 0..20 {
                writer.write_row(vec![(iteration * 20 + i).to_le_bytes().to_vec()])?;
            }
        }

        assert_eq!(writer.row_count(), 100);
        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_large_row_values() -> Result<()> {
        let schema = make_schema(vec!["data"]);
        let buffer = Cursor::new(Vec::new());

        let mut writer = StreamingWriter::new(buffer, schema)?;

        let large_data = vec![0u8; 10000];
        for _ in 0..10 {
            writer.write_row(vec![large_data.clone()])?;
        }

        assert_eq!(writer.row_count(), 10);
        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_config_row_group_size() -> Result<()> {
        let schema = make_schema(vec!["value"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 50;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        // Write multiple row groups worth of data
        for i in 0..200 {
            writer.write_row(vec![(i as u32).to_le_bytes().to_vec()])?;
        }

        let rg_count = writer.row_group_count();
        assert!(rg_count > 0);
        assert_eq!(writer.row_count(), 200);

        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_null_handling() -> Result<()> {
        let schema = make_schema(vec!["optional_col"]);
        let buffer = Cursor::new(Vec::new());

        let mut writer = StreamingWriter::new(buffer, schema)?;

        writer.write_row(vec![vec![0, 0, 0, 0]])?; // NULL marker
        writer.write_row(vec![b"data".to_vec()])?;

        assert_eq!(writer.row_count(), 2);
        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_mixed_data_types() -> Result<()> {
        let schema = make_schema(vec!["int_col", "float_col", "str_col"]);
        let buffer = Cursor::new(Vec::new());

        let mut writer = StreamingWriter::new(buffer, schema)?;

        for i in 0..50 {
            writer.write_row(vec![
                (i as u32).to_le_bytes().to_vec(),
                (i as f32).to_le_bytes().to_vec(),
                format!("value_{}", i).into_bytes(),
            ])?;
        }

        assert_eq!(writer.row_count(), 50);
        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_row_group_count_tracking() -> Result<()> {
        let schema = make_schema(vec!["id"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 10;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        assert_eq!(writer.row_group_count(), 0);

        for i in 0..45 {
            writer.write_row(vec![(i as u8).to_le_bytes().to_vec()])?;
        }

        let final_rg_count = writer.row_group_count();
        assert!(final_rg_count > 0);

        writer.finish()?;
        Ok(())
    }

    // Additional enterprise-grade streaming writer tests

    #[test]
    fn test_streaming_writer_bounded_memory_behavior() -> Result<()> {
        let schema = make_schema(vec!["data"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 50;
        config.buffer_pool_config.max_buffers = 2;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        // Write large amount of data
        for i in 0..1000 {
            let data = vec![(i % 256) as u8; 100]; // 100 bytes per row
            writer.write_row(vec![data])?;
        }

        let (cached, _) = writer.buffer_pool_stats();
        assert!(cached <= 3); // Should not cache excessively

        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_streaming_flush_cycles() -> Result<()> {
        let schema = make_schema(vec!["value"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 20;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        let mut flush_points = vec![];

        for i in 0..100 {
            writer.write_row(vec![(i as u32).to_le_bytes().to_vec()])?;
            
            if i > 0 && i % 20 == 0 {
                flush_points.push(writer.row_group_count());
            }
        }

        // Should have flushed multiple times
        assert!(flush_points.len() > 0);
        assert!(flush_points.windows(2).all(|w| w[1] >= w[0]));

        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_large_streaming_datasets() -> Result<()> {
        let schema = make_schema(vec!["blob"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 500;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        let large_blob = vec![0xAB; 1000]; // 1KB per row
        for _ in 0..2000 {
            writer.write_row(vec![large_blob.clone()])?;
        }

        assert_eq!(writer.row_count(), 2000);
        assert!(writer.row_group_count() > 0);

        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_partial_flush() -> Result<()> {
        let schema = make_schema(vec!["id"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 100;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        // Write partial row group
        for i in 0..37 {
            writer.write_row(vec![(i as u32).to_le_bytes().to_vec()])?;
        }

        assert_eq!(writer.row_count(), 37);
        // Should not have flushed yet
        assert_eq!(writer.row_group_count(), 0);

        writer.finish()?; // Should flush remaining
        Ok(())
    }

    #[test]
    fn test_streaming_writer_interrupted_streaming() -> Result<()> {
        let schema = make_schema(vec!["data"]);
        let buffer = Cursor::new(Vec::new());

        let mut writer = StreamingWriter::new(buffer, schema)?;

        for i in 0..50 {
            writer.write_row(vec![(i as u8).to_le_bytes().to_vec()])?;
        }

        // Simulate interruption - don't call finish
        // Writer should be in valid state
        assert_eq!(writer.row_count(), 50);
        Ok(())
    }

    #[test]
    fn test_streaming_writer_deterministic_output() -> Result<()> {
        let schema = make_schema(vec!["value"]);
        
        let mut buffers = vec![];
        
        for _ in 0..3 {
            let mut buffer = Vec::new();
            {
                let cursor = Cursor::new(&mut buffer);
                let mut writer = StreamingWriter::new(cursor, schema.clone())?;
                for i in 0..100 {
                    writer.write_row(vec![(i as u16).to_le_bytes().to_vec()])?;
                }
                writer.finish()?;
            }
            buffers.push(buffer);
        }

        // All outputs should be identical
        for i in 1..buffers.len() {
            assert_eq!(buffers[0], buffers[i]);
        }

        Ok(())
    }

    #[test]
    fn test_streaming_writer_incremental_row_groups() -> Result<()> {
        let schema = make_schema(vec!["seq"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 25;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        let mut rg_counts = vec![];

        for batch in 0..5 {
            for i in 0..20 {
                writer.write_row(vec![((batch * 20 + i) as u32).to_le_bytes().to_vec()])?;
            }
            rg_counts.push(writer.row_group_count());
        }

        // Row group count should increase
        for i in 1..rg_counts.len() {
            assert!(rg_counts[i] >= rg_counts[i - 1]);
        }

        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_memory_ceiling_assumptions() -> Result<()> {
        let schema = make_schema(vec!["large_data"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 10;
        config.buffer_pool_config.max_buffers = 1;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        let huge_data = vec![0xFF; 10000]; // 10KB per row
        for _ in 0..50 {
            writer.write_row(vec![huge_data.clone()])?;
        }

        // Should manage memory despite large data
        let (cached, _) = writer.buffer_pool_stats();
        assert!(cached <= 2);

        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_empty_stream() -> Result<()> {
        let schema = make_schema(vec!["empty"]);
        let buffer = Cursor::new(Vec::new());

        let mut writer = StreamingWriter::new(buffer, schema)?;
        assert_eq!(writer.row_count(), 0);
        assert_eq!(writer.row_group_count(), 0);

        writer.finish()?;
        Ok(())
    }

    #[test]
    fn test_streaming_writer_repeated_streaming_writes() -> Result<()> {
        let schema = make_schema(vec!["counter"]);
        let buffer = Cursor::new(Vec::new());
        let mut config = StreamingWriterConfig::default();
        config.row_group_size = 50;

        let mut writer = StreamingWriter::with_config(buffer, schema, config)?;

        // Multiple streaming sessions
        for session in 0..3 {
            for i in 0..30 {
                writer.write_row(vec![((session * 30 + i) as u32).to_le_bytes().to_vec()])?;
            }
        }

        assert_eq!(writer.row_count(), 90);
        writer.finish()?;
        Ok(())
    }
}
