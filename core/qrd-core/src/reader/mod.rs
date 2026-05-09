//! File reader implementation

use crate::error::Result;
use crate::footer::Footer;
use crate::schema::Schema;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// File reader for QRD format
pub struct FileReader {
    file_data: Vec<u8>,
    schema: Schema,
    row_count: u32,
    row_group_offsets: Vec<u64>,
    footer_offset: u64,
}

impl FileReader {
    /// Open a QRD file for reading
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data)?;

        if file_data.len() < 36 {
            return Err(crate::error::Error::InvalidData(
                "File too small".to_string(),
            ));
        }

        // Parse header
        let header = &file_data[0..32];

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
        let row_count = u32::from_le_bytes([header[16], header[17], header[18], header[19]]);

        // Read footer length from end
        let file_len = file_data.len();
        let footer_len_bytes = &file_data[file_len - 4..file_len];
        let footer_length = u32::from_le_bytes([
            footer_len_bytes[0],
            footer_len_bytes[1],
            footer_len_bytes[2],
            footer_len_bytes[3],
        ]) as usize;

        let footer_start = file_len - 4 - footer_length;
        let footer_data = &file_data[footer_start..footer_start + footer_length];

        let footer = Footer::deserialize(footer_data)?;

        // Verify schema ID matches
        if footer.schema.schema_id != schema_id {
            return Err(crate::error::Error::InvalidSchema(
                "Schema ID mismatch".to_string(),
            ));
        }

        Ok(FileReader {
            file_data,
            schema: footer.schema.clone(),
            row_count,
            row_group_offsets: footer.row_group_offsets,
            footer_offset: footer_start as u64,
        })
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

        let row_group_data = &self.file_data[offset..end_offset];
        RowGroup::deserialize(row_group_data)
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
            let row_group = crate::rowgroup::RowGroup::deserialize(&row_group_data)?;

            // Reassemble rows from columns
            let rows = self.reassemble_rows_from_columns(&row_group)?;
            all_rows.extend(rows);
        }

        Ok(all_rows)
    }

    /// Read specific columns
    pub fn read_columns(&self, column_indices: &[usize]) -> Result<Vec<Vec<u8>>> {
        if column_indices.is_empty() {
            return Ok(Vec::new());
        }

        let mut columns: Vec<Vec<u8>> = column_indices.iter().map(|_| Vec::new()).collect();

        for (group_index, offset) in self.row_group_offsets.iter().enumerate() {
            let row_group_data = self.read_row_group_at_offset(group_index, *offset)?;
            let row_group = crate::rowgroup::RowGroup::deserialize(&row_group_data)?;

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

                if field.field_type.fixed_size().is_none() {
                    return Err(crate::error::Error::Other(format!(
                        "Variable-length column types not yet supported: {} ({})",
                        field.name, field.field_type
                    )));
                }

                columns[selected_index].extend_from_slice(&column.encoded_data);
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

        if offset >= end_offset || offset >= self.file_data.len() {
            return Err(crate::error::Error::InvalidData(
                format!("Invalid row group offset: {}", offset)
            ));
        }

        // Read row group data
        let row_group_data = &self.file_data[offset..end_offset];
        Ok(row_group_data.to_vec())
    }

    /// Reassemble rows from column chunks
    fn reassemble_rows_from_columns(&self, row_group: &crate::rowgroup::RowGroup) -> Result<Vec<Vec<u8>>> {
        if row_group.columns.is_empty() {
            return Ok(vec![]);
        }

        let mut rows = Vec::new();

        // For each row
        for row_idx in 0..row_group.row_count as usize {
            let mut row_data = Vec::new();

            // For each column (in order)
            for (col_idx, column) in row_group.columns.iter().enumerate() {
                if col_idx >= self.schema.fields.len() {
                    return Err(crate::error::Error::InvalidSchema(
                        format!("Column index {} exceeds schema field count", col_idx)
                    ));
                }

                let field = &self.schema.fields[col_idx];
                match field.field_type.fixed_size() {
                    Some(field_size) => {
                        let start = row_idx * field_size;
                        let end = start + field_size;
                        if end <= column.encoded_data.len() {
                            row_data.extend_from_slice(&column.encoded_data[start..end]);
                        } else {
                            // Handle short reads - return error
                            return Err(crate::error::Error::InvalidData(
                                format!("Column {} data too short for row {}", col_idx, row_idx)
                            ));
                        }
                    }
                    None => {
                        return Err(crate::error::Error::Other(
                            format!("Variable-length column types not yet supported: {} ({})", 
                                   field.name, field.field_type)
                        ));
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
}
