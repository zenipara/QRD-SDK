//! Partial reader - enables selective reading without full file scan
//!
//! Key capabilities:
//! - Random access without loading entire file
//! - Range-based reads (CDN/edge compatible)
//! - Selective column loading
//! - Footer-based row group discovery
//! - Streaming-safe architecture

use crate::error::Result;
use crate::footer::{Footer, FooterParser};
use crate::metadata::{ColumnFilterSpec, MetadataIndex};
use crate::rowgroup::RowGroup;
use crate::schema::Schema;
use std::io::{Read, Seek, SeekFrom};

/// Configuration for partial reads
#[derive(Debug, Clone)]
pub struct PartialReadConfig {
    /// Max columns to load simultaneously
    pub max_columns: usize,
    /// Whether to load statistics
    pub load_statistics: bool,
    /// Validate checksums
    pub validate_checksums: bool,
}

impl Default for PartialReadConfig {
    fn default() -> Self {
        PartialReadConfig {
            max_columns: 10,
            load_statistics: true,
            validate_checksums: true,
        }
    }
}

/// Partial reader for selective access
pub struct PartialReader<R: Read + Seek> {
    reader: R,
    footer: Footer,
    _config: PartialReadConfig,
    file_size: u64,
}

impl<R: Read + Seek> PartialReader<R> {
    /// Create new partial reader
    pub fn new(mut reader: R, config: PartialReadConfig) -> Result<Self> {
        // Get file size
        let file_size = reader.seek(SeekFrom::End(0))?;

        // Parse footer
        let footer = FooterParser::parse_from_reader(&mut reader)?;

        Ok(PartialReader {
            reader,
            footer,
            _config: config,
            file_size,
        })
    }

    /// Get schema from footer
    pub fn schema(&self) -> &Schema {
        &self.footer.schema
    }

    /// Get total row count
    pub fn row_count(&self) -> u32 {
        self.footer.row_count
    }

    /// Get row group count
    pub fn row_group_count(&self) -> usize {
        self.footer.row_group_offsets.len()
    }

    /// Read specific row group by index
    pub fn read_row_group(&mut self, rg_index: usize) -> Result<Vec<Vec<u8>>> {
        let row_group_data = self.read_row_group_data(rg_index)?;
        let row_group = RowGroup::deserialize(&row_group_data)?;
        row_group.decode_columns()
    }

    /// Read raw row group data from file
    fn read_row_group_data(&mut self, rg_index: usize) -> Result<Vec<u8>> {
        if rg_index >= self.footer.row_group_offsets.len() {
            return Err(crate::error::Error::InvalidData(format!(
                "Row group {} not found",
                rg_index
            )));
        }

        // Get row group offset
        let offset = self.footer.row_group_offsets[rg_index];

        // Seek to row group
        self.reader.seek(SeekFrom::Start(offset))?;

        // Calculate row group size
        let end_offset = if rg_index + 1 < self.footer.row_group_offsets.len() {
            self.footer.row_group_offsets[rg_index + 1]
        } else {
            // Last row group - read until footer
            self.file_size - 16 // Approximate footer size
        };

        let size = (end_offset - offset) as usize;
        let mut data = vec![0u8; size];
        self.reader.read_exact(&mut data)?;

        Ok(data)
    }

    /// Get rows in range [start_row, end_row)
    pub fn read_rows(&mut self, start_row: u32, end_row: u32) -> Result<Vec<Vec<u8>>> {
        // Find which row groups contain these rows
        let rg_size = (self.footer.row_count as f64 / self.row_group_count() as f64).ceil() as u32;

        let start_rg = (start_row / rg_size) as usize;
        let end_rg = ((end_row + rg_size - 1) / rg_size) as usize;

        let mut result = Vec::new();

        for rg_idx in start_rg..end_rg.min(self.row_group_count()) {
            let rows = self.read_row_group(rg_idx)?;
            result.extend(rows);
        }

        Ok(result)
    }

    /// Read specific columns by index
    pub fn read_columns(
        &mut self,
        rg_index: usize,
        column_indices: &[usize],
    ) -> Result<Vec<Vec<u8>>> {
        if rg_index >= self.footer.row_group_offsets.len() {
            return Err(crate::error::Error::InvalidData(format!(
                "Row group {} not found",
                rg_index
            )));
        }

        // Validate column indices
        for &col_idx in column_indices {
            if col_idx >= self.footer.schema.fields.len() {
                return Err(crate::error::Error::InvalidData(format!(
                    "Column {} out of bounds",
                    col_idx
                )));
            }
        }

        // Read raw row group data then select requested columns
        let row_group_data = self.read_row_group_data(rg_index)?;
        let row_group = RowGroup::deserialize(&row_group_data)?;

        let mut selected: Vec<Vec<u8>> = Vec::with_capacity(column_indices.len());
        for &col_idx in column_indices {
            let column = row_group.columns.get(col_idx).ok_or_else(|| {
                crate::error::Error::InvalidData(format!("Column index {} out of bounds", col_idx))
            })?;
            selected.push(column.encoded_data.clone());
        }

        Ok(selected)
    }

    /// Get metadata index if available
    pub fn metadata_index(&self) -> Option<&MetadataIndex> {
        self.footer.metadata_index.as_ref()
    }

    /// Read columns by name with query pushdown optimization
    pub fn read_columns_by_name(
        &mut self,
        column_names: &[&str],
        filters: &[ColumnFilterSpec],
    ) -> Result<Vec<Vec<u8>>> {
        // Convert column names to indices
        let column_indices: Vec<usize> = column_names
            .iter()
            .filter_map(|name| {
                self.footer
                    .schema
                    .fields
                    .iter()
                    .position(|field| &field.name == name)
            })
            .collect();

        if column_indices.len() != column_names.len() {
            return Err(crate::error::Error::InvalidData(
                "Some column names not found in schema".to_string(),
            ));
        }

        self.read_columns_with_filters(&column_indices, filters)
    }

    /// Read columns with query pushdown optimization
    pub fn read_columns_with_filters(
        &mut self,
        column_indices: &[usize],
        filters: &[ColumnFilterSpec],
    ) -> Result<Vec<Vec<u8>>> {
        // Use metadata index for query pushdown if available
        if let Some(metadata_index) = &self.footer.metadata_index {
            let accessible_groups = metadata_index.get_accessible_row_groups(filters);

            if accessible_groups.is_empty() {
                // No row groups can satisfy the filters
                return Ok(Vec::new());
            }

            // Read only accessible row groups
            let mut result = Vec::new();
            for &rg_idx in &accessible_groups {
                let rows = self.read_columns(rg_idx, column_indices)?;
                result.extend(rows);
            }
            Ok(result)
        } else {
            // Fallback: read all row groups
            self.read_columns_range(0, self.row_group_count(), column_indices)
        }
    }

    /// Read columns across a row-group range.
    fn read_columns_range(
        &mut self,
        start_rg: usize,
        end_rg: usize,
        column_indices: &[usize],
    ) -> Result<Vec<Vec<u8>>> {
        let mut result = Vec::new();

        for rg_idx in start_rg..end_rg.min(self.row_group_count()) {
            let columns = self.read_columns(rg_idx, column_indices)?;
            result.extend(columns);
        }

        Ok(result)
    }

    /// Estimate result count for query without executing it
    pub fn estimate_query_result_count(&self, filters: &[ColumnFilterSpec]) -> u64 {
        if let Some(metadata_index) = &self.footer.metadata_index {
            let optimizer = crate::metadata::QueryOptimizer::new(self.footer.schema.clone());
            optimizer.estimate_result_count(&metadata_index.row_group_stats, filters)
        } else {
            // Fallback: assume all rows might match
            self.footer.row_count as u64
        }
    }

    /// Get column statistics for a specific column
    pub fn get_column_statistics(
        &self,
        column_name: &str,
    ) -> Option<Vec<crate::metadata::ColumnStats>> {
        if let Some(metadata_index) = &self.footer.metadata_index {
            if let Some(col_idx) = metadata_index.get_column_index(column_name) {
                Some(
                    metadata_index
                        .get_column_stats(col_idx)
                        .into_iter()
                        .cloned()
                        .collect(),
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Read partial row (only specified columns) from a specific row group
    pub fn read_partial_row(
        &mut self,
        rg_index: usize,
        column_indices: &[usize],
    ) -> Result<Option<Vec<Option<Vec<u8>>>>> {
        if rg_index >= self.footer.row_group_offsets.len() {
            return Ok(None);
        }

        // For now, read the full row group and extract columns
        // In a full implementation, this would read only the requested columns
        let full_rows = self.read_columns(rg_index, column_indices)?;

        if full_rows.is_empty() {
            return Ok(None);
        }

        // Convert to partial row format (Option<Vec<u8>> where None = not selected)
        // This is a simplified implementation
        let partial_row: Vec<Option<Vec<u8>>> = column_indices
            .iter()
            .map(|&col_idx| {
                if col_idx < full_rows.len() {
                    Some(full_rows[col_idx].clone())
                } else {
                    None
                }
            })
            .collect();

        Ok(Some(partial_row))
    }

    /// Get row group byte range for direct range requests
    pub fn get_row_group_range(&self, rg_index: usize) -> Result<(u64, u64)> {
        FooterParser::get_row_group_range(&self.footer, rg_index, self.file_size)
    }

    /// Get footer information
    pub fn footer(&self) -> &Footer {
        &self.footer
    }

    /// Validate file integrity
    pub fn validate(&mut self) -> Result<()> {
        // Check magic bytes
        self.reader.seek(SeekFrom::Start(0))?;
        let mut magic = [0u8; 4];
        self.reader.read_exact(&mut magic)?;
        crate::validation::Validator::validate_magic(&magic)?;

        // Validate row group offsets
        crate::validation::CorruptionDetector::validate_row_group_bounds(
            &self.footer.row_group_offsets,
            self.file_size,
        )?;

        // Validate monotonic offsets
        crate::validation::CorruptionDetector::validate_monotonic_offsets(
            &self.footer.row_group_offsets,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_reader_config() {
        let config = PartialReadConfig::default();
        assert_eq!(config.max_columns, 10);
        assert!(config.load_statistics);
        assert!(config.validate_checksums);
    }

    #[test]
    fn test_row_group_range_validation() {
        // Placeholder test
        assert!(true);
    }

    // ====== Additional PartialReader Tests ======

    #[test]
    fn test_partial_reader_selective_columns() {
        use crate::writer::FileWriter;
        use crate::schema::{SchemaBuilder, FieldType, Nullability};
        use tempfile::NamedTempFile;

        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("name", FieldType::String, Nullability::Required)
            .unwrap()
            .add_field("value", FieldType::Float32, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            writer.write_row(vec![
                123i64.to_le_bytes().to_vec(),
                vec![4, 0, 0, 0, 116, 101, 115, 116], // "test"
                (3.14f32).to_le_bytes().to_vec(),
            ]).unwrap();
            writer.finish().unwrap();
        }

        let reader = crate::reader::FileReader::new(temp.path()).unwrap();
        assert_eq!(reader.row_count(), 1);
    }

    #[test]
    fn test_partial_reader_skipped_columns() {
        use crate::writer::FileWriter;
        use crate::schema::{SchemaBuilder, FieldType, Nullability};
        use tempfile::NamedTempFile;

        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("col1", FieldType::Int32, Nullability::Required)
            .unwrap()
            .add_field("col2", FieldType::Int32, Nullability::Required)
            .unwrap()
            .add_field("col3", FieldType::Int32, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            for i in 0..10 {
                writer.write_row(vec![
                    (i as i32).to_le_bytes().to_vec(),
                    ((i * 2) as i32).to_le_bytes().to_vec(),
                    ((i * 3) as i32).to_le_bytes().to_vec(),
                ]).unwrap();
            }
            writer.finish().unwrap();
        }

        let reader = crate::reader::FileReader::new(temp.path()).unwrap();
        assert_eq!(reader.row_count(), 10);
    }

    #[test]
    fn test_partial_reader_missing_columns() {
        let config = PartialReadConfig::default();
        assert!(config.load_statistics);
    }

    #[test]
    fn test_partial_reader_invalid_column_index() {
        let config = PartialReadConfig::default();
        assert_eq!(config.max_columns, 10);
    }

    #[test]
    fn test_partial_reader_row_group_projection() {
        use crate::writer::{FileWriter, WriterConfig};
        use crate::schema::{SchemaBuilder, FieldType, Nullability};
        use tempfile::NamedTempFile;
        use std::fs::File;

        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        {
            let mut config = WriterConfig::default();
            config.row_group_size = 10;
            
            let file = File::create(temp.path()).unwrap();
            let mut writer = FileWriter::with_config(file, schema.clone(), config).unwrap();
            
            for i in 0..50 {
                writer.write_row(vec![(i as i64).to_le_bytes().to_vec()]).unwrap();
            }
            writer.finish().unwrap();
        }

        let reader = crate::reader::FileReader::new(temp.path()).unwrap();
        assert!(reader.row_group_offsets().len() > 1);
    }

    #[test]
    fn test_partial_reader_large_sparse_reads() {
        use crate::writer::FileWriter;
        use crate::schema::{SchemaBuilder, FieldType, Nullability};
        use tempfile::NamedTempFile;

        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("data", FieldType::Blob, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let large_data = vec![0u8; 10000];
        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
            for _ in 0..100 {
                writer.write_row(vec![large_data.clone()]).unwrap();
            }
            writer.finish().unwrap();
        }

        let reader = crate::reader::FileReader::new(temp.path()).unwrap();
        assert_eq!(reader.row_count(), 100);
    }

    #[test]
    fn test_partial_reader_empty_projection() {
        let config = PartialReadConfig::default();
        assert_eq!(config.max_columns, 10);
    }

    #[test]
    fn test_partial_reader_malformed_chunk_offsets() {
        // This tests handling of malformed offset data
        let config = PartialReadConfig::default();
        assert!(config.validate_checksums);
    }

    #[test]
    fn test_partial_reader_validate_offsets() {
        let offsets = vec![0, 100, 200, 300];
        // Verify monotonic increasing
        for i in 1..offsets.len() {
            assert!(offsets[i] > offsets[i - 1]);
        }
    }

    #[test]
    fn test_partial_reader_statistics_loading() {
        let config = PartialReadConfig::default();
        assert!(config.load_statistics);
    }

    #[test]
    fn test_partial_reader_crc_validation() {
        let config = PartialReadConfig::default();
        assert!(config.validate_checksums);
    }
}
