//! File writer implementation with streaming support
//!
//! Provides:
//! - FileWriter: Traditional file-based writer
//! - StreamingWriter: Bounded-memory streaming writer for large datasets
//! - Buffer pooling for memory efficiency

pub mod buffer_pool;
pub mod streaming_writer;

// Re-exports
pub use buffer_pool::{BufferPool, BufferPoolConfig};
pub use streaming_writer::{StreamingWriter, StreamingWriterConfig};

use crate::columnar::RowBuffer;
use crate::compression::CompressionLevel;
use crate::error::Result;
use crate::footer::Footer;
use crate::metadata::{RowGroupStats, MetadataIndex};
use crate::rowgroup::RowGroup;
use crate::schema::Schema;
use byteorder::{LittleEndian, WriteBytesExt};
use std::fs::File;
use std::io::{Write, Seek, SeekFrom};
use std::path::Path;

/// Configuration for the writer
#[derive(Debug, Clone)]
pub struct WriterConfig {
    /// Row group size
    pub row_group_size: u32,
    /// Compression level
    pub compression_level: u8,
}

impl Default for WriterConfig {
    fn default() -> Self {
        WriterConfig {
            row_group_size: crate::DEFAULT_ROW_GROUP_SIZE,
            compression_level: 3,
        }
    }
}

/// File writer for QRD format
pub struct FileWriter {
    file: File,
    schema: Schema,
    config: WriterConfig,
    row_buffer: RowBuffer,
    row_group_count: u32,
    total_rows: u32,
    row_group_offsets: Vec<u64>,
    current_offset: u64,
    // Statistics collection
    current_row_group_stats: RowGroupStats,
    row_group_stats: Vec<RowGroupStats>,
}

impl FileWriter {
    /// Create a new file writer
    pub fn new(path: impl AsRef<Path>, schema: Schema) -> Result<Self> {
        let file = File::create(path)?;
        Self::with_config(file, schema, WriterConfig::default())
    }

    /// Create with custom config
    pub fn with_config(mut file: File, schema: Schema, config: WriterConfig) -> Result<Self> {
        // Write file header
        file.write_all(crate::QRD_MAGIC)?;
        file.write_u16::<LittleEndian>(crate::QRD_VERSION_MAJOR)?;
        file.write_u16::<LittleEndian>(crate::QRD_VERSION_MINOR)?;
        file.write_u32::<LittleEndian>(schema.schema_id)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;
        file.write_u32::<LittleEndian>(now)?; // created_at

        // Placeholder for row count (will be updated at finish)
        file.write_u32::<LittleEndian>(0)?;
        file.write_u32::<LittleEndian>(schema.fields.len() as u32)?;
        file.write_u32::<LittleEndian>(config.row_group_size)?;
        file.write_u32::<LittleEndian>(0)?; // reserved

        let row_buffer = RowBuffer::new(schema.fields.len());

        Ok(FileWriter {
            file,
            schema,
            config,
            row_buffer,
            row_group_count: 0,
            total_rows: 0,
            row_group_offsets: Vec::new(),
            current_offset: 32, // After header
            current_row_group_stats: RowGroupStats::new(&schema),
            row_group_stats: Vec::new(),
        })
    }

    /// Write a single row (as column data)
    pub fn write_row(&mut self, row: Vec<Vec<u8>>) -> Result<()> {
        // Convert row data to Option<Vec<u8>> for statistics (empty vec = null)
        let stats_row: Vec<Option<Vec<u8>>> = row.iter()
            .map(|col| if col.is_empty() { None } else { Some(col.clone()) })
            .collect();

        // Update statistics
        self.current_row_group_stats.update_row(&stats_row);

        self.row_buffer.add_row(row)?;
        self.total_rows += 1;

        // Flush row group if threshold reached
        if self.row_buffer.row_count() >= self.config.row_group_size {
            self.flush_row_group()?;
        }

        Ok(())
    }

    /// Flush current row group to file
    fn flush_row_group(&mut self) -> Result<()> {
        if self.row_buffer.is_empty() {
            return Ok(());
        }

        // Record offset
        self.row_group_offsets.push(self.current_offset);

        // Transpose rows to columns
        let field_types: Vec<_> = self.schema.fields.iter().map(|f| f.field_type).collect();
        let columns = self.row_buffer.transpose(&field_types)?;

        // Create row group
        let mut row_group = RowGroup::new(self.row_buffer.row_count());

        // Process each column (parallel if threading enabled)
        #[cfg(feature = "threading")]
        {
            use rayon::prelude::*;
            let processed_columns: Result<Vec<_>> = columns
                .into_par_iter()
                .map(|column| {
                    let encoding = crate::encoding::select_encoding(&column.field_type, &column.data);
                    let mut temp_row_group = RowGroup::new(self.row_buffer.row_count());
                    temp_row_group.process_column(
                        column,
                        encoding,
                        crate::compression::CompressionCodec::Zstd,
                        CompressionLevel::new(self.config.compression_level),
                    )?;
                    Ok(temp_row_group.columns.into_iter().next().unwrap())
                })
                .collect();

            row_group.columns = processed_columns?;
        }

        #[cfg(not(feature = "threading"))]
        {
            // Sequential processing
            for column in columns {
                let encoding = crate::encoding::select_encoding(&column.field_type, &column.data);
                row_group.process_column(
                    column,
                    encoding,
                    crate::compression::CompressionCodec::Zstd,
                    CompressionLevel::new(self.config.compression_level),
                )?;
            }
        }

        // Serialize and write row group
        let rg_bytes = row_group.serialize()?;
        self.file.write_all(&rg_bytes)?;
        self.current_offset += rg_bytes.len() as u64;

        // Clear buffer
        self.row_buffer.clear();
        self.row_group_count += 1;

        // Save current row group statistics and reset for next group
        let completed_stats = std::mem::replace(&mut self.current_row_group_stats, RowGroupStats::new(&self.schema));
        self.row_group_stats.push(completed_stats);

        Ok(())
    }

    /// Finish writing and close the file
    pub fn finish(mut self) -> Result<()> {
        // Flush final row group
        self.flush_row_group()?;

        // Create metadata index
        let metadata_index = MetadataIndex::new(
            &self.schema,
            self.row_group_offsets.clone(),
            self.row_group_stats,
        );

        // Build footer with metadata index
        let mut footer = Footer::with_metadata_index(
            self.schema.clone(),
            self.total_rows,
            metadata_index
        );
        footer.row_group_offsets = self.row_group_offsets;

        let footer_bytes = footer.serialize()?;
        let footer_length = footer_bytes.len() as u32;

        // Write footer
        self.file.write_all(&footer_bytes)?;
        self.current_offset += footer_bytes.len() as u64;

        // Write footer length
        self.file.write_u32::<LittleEndian>(footer_length)?;

        // Update row count in header
        self.file.seek(SeekFrom::Start(16))?;
        self.file.write_u32::<LittleEndian>(self.total_rows)?;

        self.file.sync_all()?;
        Ok(())
    }

    /// Get row count
    pub fn row_count(&self) -> u32 {
        self.total_rows
    }

    /// Get schema
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Get row group count
    pub fn row_group_count(&self) -> u32 {
        self.row_group_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::FileReader;
    use crate::schema::{FieldType, Nullability, SchemaBuilder};
    use tempfile::NamedTempFile;

    fn serialize_string(s: &str) -> Vec<u8> {
        let mut result = Vec::new();
        let bytes = s.as_bytes();
        result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(bytes);
        result
    }

    #[test]
    fn test_writer_creation() {
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let writer = FileWriter::new(temp.path(), schema).unwrap();
        assert_eq!(writer.row_count(), 0);
    }

    #[test]
    fn test_round_trip() {
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("name", FieldType::String, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        // Write data
        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

            for i in 0..10 {
                let id_bytes = (i as i64).to_le_bytes().to_vec();
                let name_str = format!("user_{}", i);
                let name_bytes = serialize_string(&name_str);

                writer.write_row(vec![id_bytes, name_bytes]).unwrap();
            }

            writer.finish().unwrap();
        }

        // Read data back
        {
            let reader = FileReader::new(temp.path()).unwrap();
            assert_eq!(reader.row_count(), 10);
            assert_eq!(reader.schema().fields.len(), 2);

            // Read first row group
            let decoded_columns = reader.read_decoded_row_group(0).unwrap();
            assert_eq!(decoded_columns.len(), 2);

            // Check first column (IDs)
            let id_data = &decoded_columns[0];
            for i in 0..10 {
                let expected_id = (i as i64).to_le_bytes();
                let actual_id = &id_data[i * 8..(i + 1) * 8];
                assert_eq!(actual_id, expected_id);
            }
        }
    }

    #[test]
    fn test_writer_config() {
        let temp = NamedTempFile::new().unwrap();
        let file = File::create(temp.path()).unwrap();
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let config = WriterConfig {
            row_group_size: 100,
            compression_level: 5,
        };

        let writer = FileWriter::with_config(file, schema, config).unwrap();
        assert_eq!(writer.config.row_group_size, 100);
        assert_eq!(writer.config.compression_level, 5);
    }

    #[test]
    fn test_write_and_read_round_trip() {
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        // Write data
        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            
            // Write a row (1 column with 8 bytes for Int64)
            let row = vec![vec![1u8, 0, 0, 0, 0, 0, 0, 0]]; // 1 as little-endian i64
            writer.write_row(row).unwrap();
            
            // Write another row
            let row = vec![vec![2u8, 0, 0, 0, 0, 0, 0, 0]]; // 2 as little-endian i64
            writer.write_row(row).unwrap();
            
            writer.finish().unwrap();
        }

        // Read data back
        let reader = FileReader::new(temp.path()).unwrap();
        assert_eq!(reader.row_count(), 2);
        assert_eq!(reader.schema().fields.len(), 1);
        assert_eq!(reader.row_group_offsets().len(), 1); // Both rows should be in one row group
    }
}
