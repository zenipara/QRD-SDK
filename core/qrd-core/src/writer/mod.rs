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
use crate::encryption::EncryptionConfig;
use crate::ecc::EccConfig;
use crate::error::Result;
use crate::footer::Footer;
use crate::metadata::{RowGroupStats, MetadataIndex};
use crate::rowgroup::RowGroup;
use crate::schema::Schema;
use crate::validation::Validator;
use byteorder::{LittleEndian, WriteBytesExt};
use std::fs::File;
use std::io::Write;
use std::path::Path;

const PER_COLUMN_ROW_GROUP_MARKER: u8 = 0xFF;

/// Configuration for the writer
#[derive(Debug, Clone)]
pub struct WriterConfig {
    /// Row group size
    pub row_group_size: u32,
    /// Compression level
    pub compression_level: u8,
    /// Encryption configuration (None = no encryption)
    pub encryption: Option<EncryptionConfig>,
    /// ECC configuration (None = no error correction)
    pub ecc: Option<EccConfig>,
    /// Whether to encrypt footer (default: true if encryption is enabled)
    pub encrypt_footer: bool,
    /// Enable per-column encryption (default: false)
    pub per_column_encryption: bool,
}

impl Default for WriterConfig {
    fn default() -> Self {
        WriterConfig {
            row_group_size: crate::DEFAULT_ROW_GROUP_SIZE,
            compression_level: 3,
            encryption: None,
            ecc: None,
            encrypt_footer: true,
            per_column_encryption: false,
        }
    }
}

/// Helper function to write QRD file header
/// This is shared between FileWriter and StreamingWriter to avoid duplication
pub(crate) fn write_header(writer: &mut dyn Write, schema: &Schema, row_group_size: u32) -> Result<()> {
    crate::utils::write_header(writer, schema, row_group_size)
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
        let current_row_group_stats = RowGroupStats::new(&schema);

        // Write file header using shared helper
        write_header(&mut file, &schema, config.row_group_size)?;

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
            current_row_group_stats,
            row_group_stats: Vec::new(),
        })
    }

    /// Write a single row (as column data)
    pub fn write_row(&mut self, row: Vec<Vec<u8>>) -> Result<()> {
        let normalized_row: Vec<Vec<u8>> = row
            .iter()
            .enumerate()
            .map(|(col_idx, col)| {
                let field = &self.schema.fields[col_idx];

                if col.is_empty()
                    && field.nullability == crate::schema::Nullability::Optional
                    && field.field_type.fixed_size().is_none()
                {
                    vec![0, 0, 0, 0]
                } else {
                    col.clone()
                }
            })
            .collect();

        // Convert original row data to Option<Vec<u8>> for statistics (empty vec = null)
        let stats_row: Vec<Option<Vec<u8>>> = row.iter()
            .map(|col| if col.is_empty() { None } else { Some(col.clone()) })
            .collect();

        // Update statistics
        self.current_row_group_stats.update_row(&stats_row);

        self.row_buffer.add_row(normalized_row)?;
        self.total_rows += 1;

        // Flush row group if threshold reached
        if self.row_buffer.row_count() >= self.config.row_group_size {
            self.flush_row_group()?;
        }

        Ok(())
    }

    /// Encrypt row group using per-column keys derived from master key
    /// 
    /// Each column is encrypted with a unique key derived from the master key
    /// and the column name, allowing selective decryption of specific columns.
    fn encrypt_row_group_per_column(
        &self,
        row_group: &RowGroup,
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

        // Capture statistics for this row group and attach to row_group before serialization
        let stats_for_group = self.current_row_group_stats.clone();
        row_group.column_stats = Some(stats_for_group.column_stats.clone());

        // Serialize and write row group with security pipeline
        let rg_bytes = row_group.serialize()?;
        
        // STEP 1: Per-column or master encryption (if enabled)
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
        let final_bytes = if let Some(ref ecc_config) = self.config.ecc {
            let wrapped_bytes = self.wrap_row_group_with_crc(&encrypted_bytes)?;
            let mut codec = crate::ecc::EccCodec::new(ecc_config.clone())?;
            let encoded = codec.encode(&wrapped_bytes)?;
            // Serialize EccEncodedData to bytes
            encoded.to_bytes()?
        } else {
            encrypted_bytes
        };

        // Write to file
        self.file.write_all(&final_bytes)?;
        self.current_offset += final_bytes.len() as u64;

        // Clear buffer
        self.row_buffer.clear();
        self.row_group_count += 1;

        // Save current row group statistics and reset for next group
        let completed_stats = std::mem::replace(&mut self.current_row_group_stats, RowGroupStats::new(&self.schema));
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

        // CRC computed over serialized footer bytes
        
        // Encrypt footer if enabled and configured
        let final_footer_bytes = if self.config.encrypt_footer {
            if let Some(ref enc_config) = self.config.encryption {
                crate::encryption::encrypt(&footer_bytes, enc_config)?
            } else {
                footer_bytes.clone()
            }
        } else {
            footer_bytes.clone()
        };

        let footer_length = final_footer_bytes.len() as u32;

        // Write footer
        self.file.write_all(&final_footer_bytes)?;
        self.current_offset += final_footer_bytes.len() as u64;

        // Write footer length
        self.file.write_u32::<LittleEndian>(footer_length)?;

        // We store the authoritative row_count in the footer and use a sentinel
        // in the header for compatibility with non-seekable writers. No seek/patch required.
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
    fn test_per_column_encryption_round_trip() {
        use crate::encryption::EncryptionConfig;
        use std::fs::File;

        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("secret", FieldType::String, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let key = EncryptionConfig::generate_key();
        let mut config = WriterConfig::default();
        config.encryption = Some(EncryptionConfig::new(key.clone()).unwrap());
        config.per_column_encryption = true;
        config.encrypt_footer = false;

        let file = File::create(temp.path()).unwrap();
        let mut writer = FileWriter::with_config(file, schema.clone(), config).unwrap();

        for i in 0..3 {
            let id_bytes = (i as i64).to_le_bytes().to_vec();
            let secret = format!("secret_{}", i);
            let secret_bytes = serialize_string(&secret);
            writer.write_row(vec![id_bytes, secret_bytes]).unwrap();
        }
        writer.finish().unwrap();

        let reader = FileReader::with_decryption(temp.path(), EncryptionConfig::new(key).unwrap()).unwrap();
        assert_eq!(reader.row_count(), 3);
        let decoded_columns = reader.read_decoded_row_group(0).unwrap();
        assert_eq!(decoded_columns.len(), 2);

        let id_data = &decoded_columns[0];
        for i in 0..3 {
            let expected_id = (i as i64).to_le_bytes();
            let actual_id = &id_data[i * 8..(i + 1) * 8];
            assert_eq!(actual_id, expected_id);
        }

        let secret_data = &decoded_columns[1];
        let expected_secret = serialize_string("secret_0");
        assert_eq!(secret_data[0..expected_secret.len()], expected_secret[..]);
    }

    #[test]
    fn test_ecc_row_group_checksum_detects_silent_corruption() {
        use crate::ecc::EccConfig;
        use std::fs::File;

        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("value", FieldType::String, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let mut config = WriterConfig::default();
        config.ecc = Some(EccConfig::with_chunk_size(2, 16).unwrap());
        config.row_group_size = 2;

        let file = File::create(temp.path()).unwrap();
        let mut writer = FileWriter::with_config(file, schema.clone(), config.clone()).unwrap();
        for i in 0..2 {
            let id_bytes = (i as i64).to_le_bytes().to_vec();
            let val_bytes = serialize_string(&format!("value_{}", i));
            writer.write_row(vec![id_bytes, val_bytes]).unwrap();
        }
        writer.finish().unwrap();

        let mut raw = std::fs::read(temp.path()).unwrap();
        if raw.len() > 40 {
            raw[40] ^= 0xFF;
            std::fs::write(temp.path(), &raw).unwrap();
        }

        let reader = FileReader::with_security(temp.path(), None, config.ecc.clone()).unwrap();
        assert!(reader.read_row_group(0).is_err(), "Corrupted ECC-protected row group should be rejected by checksum verification");
    }

    #[test]
    fn test_column_statistics_null_count_roundtrip() {
        use tempfile::NamedTempFile;
        
        let temp = NamedTempFile::new().unwrap();

        let schema = crate::schema::SchemaBuilder::new()
            .add_field("opt", crate::schema::FieldType::String, crate::schema::Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();

        // Write 10 rows, with 3 nulls
        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            for i in 0..10 {
                if i == 1 || i == 4 || i == 7 {
                    writer.write_row(vec![vec![]]).unwrap(); // null represented as empty
                } else {
                    let s = format!("row_{}", i);
                    let mut v = Vec::new(); v.extend_from_slice(&(s.len() as u32).to_le_bytes()); v.extend_from_slice(s.as_bytes());
                    writer.write_row(vec![v]).unwrap();
                }
            }
            writer.finish().unwrap();
        }

        // Read file bytes and parse footer to inspect metadata index
        let raw = std::fs::read(temp.path()).unwrap();
        let len = raw.len();
        let footer_len = u32::from_le_bytes([raw[len - 4], raw[len - 3], raw[len - 2], raw[len - 1]]) as usize;
        let footer_start = len - 4 - footer_len;
        let footer_bytes = &raw[footer_start..footer_start + footer_len];
        let footer = crate::footer::Footer::deserialize(footer_bytes).unwrap();

        assert!(footer.metadata_index.is_some());
        let mi = footer.metadata_index.unwrap();
        assert_eq!(mi.row_group_stats.len(), 1);
        let col_stats = &mi.row_group_stats[0].column_stats[0];
        assert_eq!(col_stats.null_count as usize, 3);
    }

    #[test]
    fn test_column_statistics_min_max_roundtrip() {
        use tempfile::NamedTempFile;
        let temp = NamedTempFile::new().unwrap();

        let schema = crate::schema::SchemaBuilder::new()
            .add_field("v", crate::schema::FieldType::Int32, crate::schema::Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        // Write values 1..=100
        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            for i in 1..=100 {
                writer.write_row(vec![(i as i32).to_le_bytes().to_vec()]).unwrap();
            }
            writer.finish().unwrap();
        }

        let raw = std::fs::read(temp.path()).unwrap();
        let len = raw.len();
        // Footer format: [footer_bytes][footer_length: u32]
        // footer_length is the last 4 bytes
        let footer_len = u32::from_le_bytes([raw[len - 4], raw[len - 3], raw[len - 2], raw[len - 1]]) as usize;
        let footer_start = len - 4 - footer_len;
        let footer_bytes = &raw[footer_start..footer_start + footer_len];
        let footer = crate::footer::Footer::deserialize(footer_bytes).unwrap();

        let mi = footer.metadata_index.expect("metadata index present");
        let col_stats = &mi.row_group_stats[0].column_stats[0];
        // min_value and max_value are serialized bytes — decode int32 little-endian
        let min_bytes = col_stats.min_value.as_ref().unwrap();
        let max_bytes = col_stats.max_value.as_ref().unwrap();
        let min = i32::from_le_bytes([min_bytes[0], min_bytes[1], min_bytes[2], min_bytes[3]]);
        let max = i32::from_le_bytes([max_bytes[0], max_bytes[1], max_bytes[2], max_bytes[3]]);
        assert_eq!(min, 1);
        assert_eq!(max, 100);
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

        let mut config = WriterConfig::default();
        config.row_group_size = 100;
        config.compression_level = 5;

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
