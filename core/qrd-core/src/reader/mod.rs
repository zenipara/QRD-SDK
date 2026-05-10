//! File reader implementation
//!
//! Provides:
//! - FileReader: Traditional full-file reader
//! - PartialReader: Random access without loading entire file
//! - RangeReader: HTTP byte-range compatible reads

pub mod partial_reader;
pub mod range_reader;

pub use partial_reader::{PartialReader, PartialReadConfig};
pub use range_reader::{ByteRange, RangeReader};

use crate::encryption::EncryptionConfig;
use crate::ecc::EccConfig;
use crate::error::Result;
use crate::footer::Footer;
use crate::rowgroup::RowGroup;
use crate::schema::Schema;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::validation::Validator;
use memmap2::{Mmap, MmapOptions};
use std::fs::File;
use std::io::{Read, Cursor};
use std::path::Path;
use std::sync::Arc;

const PER_COLUMN_ROW_GROUP_MARKER: u8 = 0xFF;

/// Backing storage for a QRD file.
#[derive(Debug)]
enum FileData {
    /// Memory-mapped file contents.
    Mapped(Mmap),
    /// In-memory file contents.
    InMemory(Vec<u8>),
}

impl FileData {
    /// View the backing file contents as a byte slice.
    fn as_slice(&self) -> &[u8] {
        match self {
            FileData::Mapped(mapped) => mapped.as_ref(),
            FileData::InMemory(buffer) => buffer.as_slice(),
        }
    }
}

/// File reader for QRD format
pub struct FileReader {
    file_data: Arc<FileData>,
    schema: Schema,
    row_count: u32,
    row_group_offsets: Vec<u64>,
    footer_offset: u64,
    // Security configurations for decryption/recovery
    encryption_config: Option<EncryptionConfig>,
    ecc_config: Option<EccConfig>,
}

impl FileReader {
    /// Open a QRD file using in-memory loading.
    pub fn open_in_memory(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data)?;

        Self::from_file_data(Arc::new(FileData::InMemory(file_data)))
    }

    /// Open a QRD file using memory-mapped storage for large files.
    #[allow(unsafe_code)]
    pub fn open_mmap(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;

        // SAFETY: The file descriptor remains valid for the duration of the
        // mapping creation call, and the returned `Mmap` owns the mapping.
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        Self::from_file_data(Arc::new(FileData::Mapped(mmap)))
    }

    /// Open a QRD file for reading.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        const MMAP_THRESHOLD: u64 = 64 * 1024 * 1024;

        if std::fs::metadata(path_ref)?.len() >= MMAP_THRESHOLD {
            Self::open_mmap(path_ref)
        } else {
            Self::open_in_memory(path_ref)
        }
    }

    /// Build a reader from loaded file contents.
    fn from_file_data(file_data: Arc<FileData>) -> Result<Self> {
        let file_slice = file_data.as_slice();

        if file_slice.len() < 36 {
            return Err(crate::error::Error::InvalidData(
                "File too small".to_string(),
            ));
        }

        // Parse header
        let header = &file_slice[0..32];

        // Verify magic
        let magic = &header[0..4];
        if magic != crate::QRD_MAGIC {
            return Err(crate::error::Error::InvalidMagic);
        }

        let version_major = u16::from_le_bytes([header[4], header[5]]);
        let version_minor = u16::from_le_bytes([header[6], header[7]]);
        
        if version_major != crate::QRD_VERSION_MAJOR || version_minor != crate::QRD_VERSION_MINOR {
            return Err(crate::error::Error::UnsupportedVersion {
                major: version_major,
                minor: version_minor,
            });
        }

        let schema_id = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        let _created_at = u32::from_le_bytes([header[12], header[13], header[14], header[15]]);
        let header_row_count = u32::from_le_bytes([header[16], header[17], header[18], header[19]]);

        // Read footer length from end
        let file_len = file_slice.len();
        let footer_len_bytes = &file_slice[file_len - 4..file_len];
        let footer_length = u32::from_le_bytes([
            footer_len_bytes[0],
            footer_len_bytes[1],
            footer_len_bytes[2],
            footer_len_bytes[3],
        ]) as usize;

        // Validate footer_length against file length to avoid underflow and truncated reads
        if footer_length > file_len - 4 {
            return Err(crate::error::Error::InvalidData(
                "Footer length larger than file size".to_string(),
            ));
        }

        let footer_start = file_len - 4 - footer_length;
        let footer_data = &file_slice[footer_start..footer_start + footer_length];

        let footer = Footer::deserialize(footer_data)?;

        // Verify schema ID matches
        if footer.schema.schema_id != schema_id {
            return Err(crate::error::Error::InvalidSchema(
                format!(
                    "Schema ID mismatch: header schema_id={:#x} but footer schema_id={:#x}. \
                     This indicates the file was created with a different schema than expected.",
                    schema_id, footer.schema.schema_id
                ),
            ));
        }

        let row_count = if header_row_count == u32::MAX {
            footer.row_count
        } else {
            header_row_count
        };

        Ok(FileReader {
            file_data,
            schema: footer.schema.clone(),
            row_count,
            row_group_offsets: footer.row_group_offsets,
            footer_offset: footer_start as u64,
            encryption_config: None,
            ecc_config: None,
        })
    }

    /// Open file with encryption support
    pub fn with_decryption(path: impl AsRef<Path>, enc_config: EncryptionConfig) -> Result<Self> {
        let mut reader = Self::new(path)?;
        reader.encryption_config = Some(enc_config);
        Ok(reader)
    }

    /// Open file with ECC recovery support
    pub fn with_ecc(path: impl AsRef<Path>, ecc_config: EccConfig) -> Result<Self> {
        let mut reader = Self::new(path)?;
        reader.ecc_config = Some(ecc_config);
        Ok(reader)
    }

    /// Open file with both encryption and ECC support
    pub fn with_security(
        path: impl AsRef<Path>,
        enc_config: Option<EncryptionConfig>,
        ecc_config: Option<EccConfig>,
    ) -> Result<Self> {
        let mut reader = Self::new(path)?;
        reader.encryption_config = enc_config;
        reader.ecc_config = ecc_config;
        Ok(reader)
    }

    /// Get schema
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Get total row count
    pub fn row_count(&self) -> u32 {
        self.row_count
    }

    /// Get row group offsets
    pub fn row_group_offsets(&self) -> &[u64] {
        &self.row_group_offsets
    }

    /// Read a specific row group
    pub fn read_row_group(&self, index: usize) -> Result<RowGroup> {
        if index >= self.row_group_offsets.len() {
            return Err(crate::error::Error::InvalidData(
                "Row group index out of bounds".to_string(),
            ));
        }

        let offset = self.row_group_offsets[index] as usize;
        let end_offset = if index + 1 < self.row_group_offsets.len() {
            self.row_group_offsets[index + 1] as usize
        } else {
            self.footer_offset as usize
        };

        let row_group_data_raw = &self.file_data.as_slice()[offset..end_offset];
        let row_group_data = self.decrypt_and_recover_row_group(row_group_data_raw)?;
        RowGroup::deserialize(&row_group_data)
    }

    /// Decrypt and recover row group data from storage bytes
    /// 
    /// Reverse pipeline:
    /// storage bytes → ECC recovery (if enabled) → decryption (if enabled) → deserialized bytes
    fn decrypt_and_recover_row_group(&self, raw_bytes: &[u8]) -> Result<Vec<u8>> {
        // STEP 1: ECC recovery (if enabled)
        let ecc_recovered = if let Some(ref ecc_config) = self.ecc_config {
            let encoded_data = crate::ecc::EccEncodedData::from_bytes(raw_bytes)?;
            let recovered = crate::ecc::decode_and_recover(&encoded_data, ecc_config)?;
            self.verify_and_strip_row_group_crc(&recovered)?
        } else {
            raw_bytes.to_vec()
        };

        // STEP 2: Decryption (if enabled)
        let decrypted = if let Some(ref enc_config) = self.encryption_config {
            if let Some(&PER_COLUMN_ROW_GROUP_MARKER) = ecc_recovered.first() {
                self.decrypt_per_column_row_group(&ecc_recovered, enc_config)?
            } else {
                crate::encryption::decrypt(&ecc_recovered, enc_config)?
            }
        } else {
            ecc_recovered
        };

        Ok(decrypted)
    }

    fn verify_and_strip_row_group_crc(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 4 {
            return Err(crate::error::Error::InvalidData(
                "Row group data too short for CRC32 wrapper".to_string(),
            ));
        }

        let expected_crc = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let payload = &data[4..];
        Validator::verify_crc32(payload, expected_crc)?;
        Ok(payload.to_vec())
    }

    fn decrypt_per_column_row_group(
        &self,
        data: &[u8],
        enc_config: &EncryptionConfig,
    ) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(data);
        let marker = cursor.read_u8()?;
        if marker != PER_COLUMN_ROW_GROUP_MARKER {
            return Err(crate::error::Error::InvalidData(
                "Unexpected row group marker for per-column encrypted row group".to_string(),
            ));
        }

        let row_count = cursor.read_u32::<LittleEndian>()?;
        let column_count = cursor.read_u16::<LittleEndian>()? as usize;

        let mut total_uncompressed = 0u32;
        let mut total_compressed = 0u32;
        let mut decrypted_chunks: Vec<Vec<u8>> = Vec::with_capacity(column_count);

        for column_index in 0..column_count {
            let encrypted_len = cursor.read_u32::<LittleEndian>()? as usize;
            let mut encrypted_chunk = vec![0u8; encrypted_len];
            cursor.read_exact(&mut encrypted_chunk)?;

            let column_name = self.schema.fields.get(column_index).ok_or_else(|| {
                crate::error::Error::InvalidSchema(format!(
                    "Schema missing column at index {} while decrypting per-column row group",
                    column_index
                ))
            })?;

            let derived_key = enc_config.derive_column_key(&column_name.name)?;
            let column_config = EncryptionConfig::new(derived_key)?;
            let chunk_bytes = crate::encryption::decrypt(&encrypted_chunk, &column_config)?;

            let mut chunk_cursor = Cursor::new(&chunk_bytes);
            let mut id_buf = [0u8; 1];
            chunk_cursor.read_exact(&mut id_buf)?;
            chunk_cursor.read_exact(&mut id_buf)?;
            let encoded_len = chunk_cursor.read_u32::<LittleEndian>()? as u32;
            let compressed_len = chunk_cursor.read_u32::<LittleEndian>()? as u32;
            total_uncompressed = total_uncompressed.saturating_add(encoded_len);
            total_compressed = total_compressed.saturating_add(compressed_len);

            decrypted_chunks.push(chunk_bytes);
        }

        let mut result = Vec::new();
        result.write_u32::<LittleEndian>(row_count)?;
        result.write_u32::<LittleEndian>(total_uncompressed)?;
        result.write_u32::<LittleEndian>(total_compressed)?;
        result.write_u16::<LittleEndian>(column_count as u16)?;

        for chunk in decrypted_chunks {
            result.extend_from_slice(&chunk);
        }

        Ok(result)
    }

    /// Read all row groups
    pub fn read_all_row_groups(&self) -> Result<Vec<RowGroup>> {
        let mut row_groups = Vec::with_capacity(self.row_group_offsets.len());
        for i in 0..self.row_group_offsets.len() {
            row_groups.push(self.read_row_group(i)?);
        }
        Ok(row_groups)
    }

    /// Get decoded data for a specific row group
    pub fn read_decoded_row_group(&self, index: usize) -> Result<Vec<Vec<u8>>> {
        let row_group = self.read_row_group(index)?;
        row_group.decode_columns()
    }

    /// Iterate over rows
    pub fn rows(&self) -> Result<Vec<Vec<u8>>> {
        let mut all_rows = Vec::new();

        for (group_index, offset) in self.row_group_offsets.iter().enumerate() {
            // Read row group data
            let row_group_data = self.read_row_group_at_offset(group_index, *offset)?;

            // Parse row group
            let row_group = RowGroup::deserialize(&row_group_data)?;

            // Reassemble rows from columns
            let rows = self.reassemble_rows_from_columns(&row_group)?;
            all_rows.extend(rows);
        }

        Ok(all_rows)
    }

    /// Read specific columns.
    ///
    /// Variable-length columns are returned in their raw serialized form,
    /// including the 4-byte little-endian length prefix.
    pub fn read_columns(&self, column_indices: &[usize]) -> Result<Vec<Vec<u8>>> {
        self.read_columns_with_length_prefix(column_indices, true)
    }

    /// Read specific columns with control over whether variable-length values
    /// keep their 4-byte length prefix in the returned bytes.
    pub fn read_columns_with_length_prefix(
        &self,
        column_indices: &[usize],
        include_length_prefix: bool,
    ) -> Result<Vec<Vec<u8>>> {
        if column_indices.is_empty() {
            return Ok(Vec::new());
        }

        let mut columns: Vec<Vec<u8>> = column_indices.iter().map(|_| Vec::new()).collect();

        for (group_index, offset) in self.row_group_offsets.iter().enumerate() {
            let row_group_data = self.read_row_group_at_offset(group_index, *offset)?;
            let row_group = RowGroup::deserialize(&row_group_data)?;

            for (selected_index, column_index) in column_indices.iter().enumerate() {
                let column = row_group.columns.get(*column_index).ok_or_else(|| {
                    crate::error::Error::InvalidSchema(format!(
                        "Column index {} out of bounds for row group",
                        column_index
                    ))
                })?;

                let field = self.schema.fields.get(*column_index).ok_or_else(|| {
                    crate::error::Error::InvalidSchema(format!(
                        "Column index {} out of bounds for schema",
                        column_index
                    ))
                })?;

                match field.field_type.fixed_size() {
                    Some(_) => columns[selected_index].extend_from_slice(&column.encoded_data),
                    None => {
                        if include_length_prefix {
                            columns[selected_index].extend_from_slice(&column.encoded_data);
                        } else {
                            let mut offset = 0usize;
                            for row_idx in 0..row_group.row_count as usize {
                                let len_end = offset.checked_add(4).ok_or_else(|| {
                                    crate::error::Error::InvalidData(format!(
                                        "Length prefix overflow for field {} ({}) at row {}",
                                        field.name, field.field_type, row_idx
                                    ))
                                })?;

                                let len_bytes = column.encoded_data.get(offset..len_end).ok_or_else(|| {
                                    crate::error::Error::InvalidData(format!(
                                        "Length prefix out of bounds for field {} ({}) at row {}",
                                        field.name, field.field_type, row_idx
                                    ))
                                })?;
                                let len = u32::from_le_bytes([
                                    len_bytes[0],
                                    len_bytes[1],
                                    len_bytes[2],
                                    len_bytes[3],
                                ]) as usize;

                                let value_start = len_end;
                                let value_end = value_start.checked_add(len).ok_or_else(|| {
                                    crate::error::Error::InvalidData(format!(
                                        "Variable-length data overflow for field {} ({}) at row {}",
                                        field.name, field.field_type, row_idx
                                    ))
                                })?;

                                let value = column.encoded_data.get(value_start..value_end).ok_or_else(|| {
                                    crate::error::Error::InvalidData(format!(
                                        "Variable-length data out of bounds for field {} ({}) at row {}",
                                        field.name, field.field_type, row_idx
                                    ))
                                })?;

                                columns[selected_index].extend_from_slice(value);
                                offset = value_end;
                            }
                        }
                    }
                }
            }
        }

        Ok(columns)
    }

    /// Read row group data from file at given offset
    fn read_row_group_at_offset(&self, group_index: usize, offset: u64) -> Result<Vec<u8>> {
        let offset = offset as usize;

        let end_offset = self
            .row_group_offsets
            .get(group_index + 1)
            .map(|next_offset| *next_offset as usize)
            .unwrap_or(self.footer_offset as usize);

        if offset >= end_offset || offset >= self.file_data.as_slice().len() {
            return Err(crate::error::Error::InvalidData(
                format!("Invalid row group offset: {}", offset)
            ));
        }

        // Read row group data (encrypted/ECC encoded if applicable)
        let row_group_data_raw = &self.file_data.as_slice()[offset..end_offset];
        
        // Decrypt and recover if needed
        self.decrypt_and_recover_row_group(row_group_data_raw)
    }

    /// Reassemble rows from column chunks
    fn reassemble_rows_from_columns(&self, row_group: &RowGroup) -> Result<Vec<Vec<u8>>> {
        if row_group.columns.is_empty() {
            return Ok(vec![]);
        }

        let mut rows = Vec::new();
        let decoded_columns = row_group.decode_columns()?;
        let row_count = row_group.row_count as usize;
        let mut col_offsets = vec![0usize; row_group.columns.len()];

        for row_idx in 0..row_count {
            let mut row_data = Vec::new();

            for (col_idx, _column) in row_group.columns.iter().enumerate() {
                if col_idx >= self.schema.fields.len() {
                    return Err(crate::error::Error::InvalidSchema(
                        format!(
                            "Column index {} exceeds schema field count for row group field set",
                            col_idx
                        )
                    ));
                }

                let field = &self.schema.fields[col_idx];
                let decoded_column = &decoded_columns[col_idx];
                match field.field_type.fixed_size() {
                    Some(field_size) => {
                        let start = row_idx.checked_mul(field_size).ok_or_else(|| {
                            crate::error::Error::InvalidData(format!(
                                "Fixed-size offset overflow for field {} ({}) at row {}",
                                field.name, field.field_type, row_idx
                            ))
                        })?;
                        let end = start.checked_add(field_size).ok_or_else(|| {
                            crate::error::Error::InvalidData(format!(
                                "Fixed-size offset overflow for field {} ({}) at row {}",
                                field.name, field.field_type, row_idx
                            ))
                        })?;
                        let slice = decoded_column.get(start..end).ok_or_else(|| {
                            crate::error::Error::InvalidData(format!(
                                "Column {} ({}) data too short for row {}",
                                field.name, field.field_type, row_idx
                            ))
                        })?;
                        row_data.extend_from_slice(slice);
                    }
                    None => {
                        let offset = col_offsets[col_idx];
                        let len_end = offset.checked_add(4).ok_or_else(|| {
                            crate::error::Error::InvalidData(format!(
                                "Length prefix overflow for field {} ({}) at row {}",
                                field.name, field.field_type, row_idx
                            ))
                        })?;

                        let len_bytes = decoded_column.get(offset..len_end).ok_or_else(|| {
                            crate::error::Error::InvalidData(format!(
                                "Length prefix out of bounds for field {} ({}) at row {}",
                                field.name, field.field_type, row_idx
                            ))
                        })?;

                        let len = u32::from_le_bytes([
                            len_bytes[0],
                            len_bytes[1],
                            len_bytes[2],
                            len_bytes[3],
                        ]) as usize;
                        let value_start = len_end;
                        let value_end = value_start.checked_add(len).ok_or_else(|| {
                            crate::error::Error::InvalidData(format!(
                                "Variable-length data overflow for field {} ({}) at row {}",
                                field.name, field.field_type, row_idx
                            ))
                        })?;

                        let value = decoded_column.get(value_start..value_end).ok_or_else(|| {
                            crate::error::Error::InvalidData(format!(
                                "Variable-length data out of bounds for field {} ({}) at row {}",
                                field.name, field.field_type, row_idx
                            ))
                        })?;

                        row_data.extend_from_slice(len_bytes);
                        row_data.extend_from_slice(value);
                        col_offsets[col_idx] = value_end;
                    }
                }
            }

            rows.push(row_data);
        }

        Ok(rows)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FieldType, Nullability, SchemaBuilder};
    use crate::writer::FileWriter;
    use tempfile::NamedTempFile;

    fn serialize_string(value: &str) -> Vec<u8> {
        let mut result = Vec::new();
        let bytes = value.as_bytes();
        result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(bytes);
        result
    }

    #[test]
    fn test_reader_error_on_missing_file() {
        let result = FileReader::new("/nonexistent/file.qrd");
        assert!(result.is_err());
    }

    #[test]
    fn test_reader_partial_column_read() {
        let temp = NamedTempFile::new().unwrap();

        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("value", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            writer.write_row(vec![
                vec![1u8, 0, 0, 0, 0, 0, 0, 0],
                vec![10u8, 0, 0, 0, 0, 0, 0, 0],
            ]).unwrap();
            writer.write_row(vec![
                vec![2u8, 0, 0, 0, 0, 0, 0, 0],
                vec![20u8, 0, 0, 0, 0, 0, 0, 0],
            ]).unwrap();
            writer.finish().unwrap();
        }

        let reader = FileReader::new(temp.path()).unwrap();
        let columns = reader.read_columns(&[0, 1]).unwrap();

        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0].len(), 16);
        assert_eq!(columns[1].len(), 16);
        assert_eq!(reader.row_count(), 2);
        assert_eq!(reader.schema().fields.len(), 2);
    }

    #[test]
    fn test_string_column_roundtrip() {
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("name", FieldType::String, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let names = vec![
            "alpha",
            "",
            "こんにちは",
            "emoji-🚀",
            "plain-ascii",
            "multi\nline",
            "with spaces",
            "ümlaut",
            "symbols-!@#$",
            "final",
        ];

        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            for (idx, name) in names.iter().enumerate() {
                writer.write_row(vec![
                    (idx as i64).to_le_bytes().to_vec(),
                    serialize_string(name),
                ]).unwrap();
            }
            writer.finish().unwrap();
        }

        let reader = FileReader::new(temp.path()).unwrap();
        let rows = reader.rows().unwrap();

        assert_eq!(rows.len(), names.len());
        for (idx, row) in rows.iter().enumerate() {
            let id = i64::from_le_bytes(row[0..8].try_into().unwrap());
            assert_eq!(id, idx as i64);

            let name_len = u32::from_le_bytes(row[8..12].try_into().unwrap()) as usize;
            let name_bytes = &row[12..12 + name_len];
            assert_eq!(name_bytes, names[idx].as_bytes());
        }

        let mixed_columns = reader.read_columns(&[0, 1]).unwrap();
        assert_eq!(mixed_columns[0].len(), 10 * 8);
        assert!(mixed_columns[1].windows(4).any(|window| window == [0, 0, 0, 0]));
    }

    #[test]
    fn test_blob_column_roundtrip() {
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("blob", FieldType::Blob, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let blobs = vec![
            vec![0u8, 1, 2, 3, 4],
            vec![0u8, 0, 1, 0, 2, 0, 3],
            vec![255u8, 0, 128, 64],
            vec![],
        ];

        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            for (idx, blob) in blobs.iter().enumerate() {
                let mut payload = Vec::new();
                payload.extend_from_slice(&(blob.len() as u32).to_le_bytes());
                payload.extend_from_slice(blob);

                writer.write_row(vec![
                    (idx as i64).to_le_bytes().to_vec(),
                    payload,
                ]).unwrap();
            }
            writer.finish().unwrap();
        }

        let reader = FileReader::new(temp.path()).unwrap();
        let rows = reader.rows().unwrap();

        assert_eq!(rows.len(), blobs.len());
        for (idx, row) in rows.iter().enumerate() {
            let id = i64::from_le_bytes(row[0..8].try_into().unwrap());
            assert_eq!(id, idx as i64);

            let blob_len = u32::from_le_bytes(row[8..12].try_into().unwrap()) as usize;
            let blob_bytes = &row[12..12 + blob_len];
            assert_eq!(blob_bytes, blobs[idx].as_slice());
        }
    }

    #[test]
    fn test_large_string_roundtrip() {
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("payload", FieldType::String, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let large_string = "a".repeat(70_000);

        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            writer.write_row(vec![serialize_string(&large_string)]).unwrap();
            writer.finish().unwrap();
        }

        let reader = FileReader::new(temp.path()).unwrap();
        let rows = reader.rows().unwrap();

        assert_eq!(rows.len(), 1);
        let len = u32::from_le_bytes(rows[0][0..4].try_into().unwrap()) as usize;
        assert_eq!(len, 70_000);
        assert_eq!(&rows[0][4..4 + len], large_string.as_bytes());
    }

    #[test]
    fn test_empty_string_roundtrip() {
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("payload", FieldType::String, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            writer.write_row(vec![serialize_string("")]).unwrap();
            writer.finish().unwrap();
        }

        let reader = FileReader::new(temp.path()).unwrap();
        let rows = reader.rows().unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(&rows[0], &[0, 0, 0, 0]);
    }

    #[test]
    fn test_null_optional_string() {
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("payload", FieldType::String, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();

        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            writer.write_row(vec![Vec::new()]).unwrap();
            writer.finish().unwrap();
        }

        let reader = FileReader::new(temp.path()).unwrap();
        let rows = reader.rows().unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(&rows[0], &[0, 0, 0, 0]);
    }
}
