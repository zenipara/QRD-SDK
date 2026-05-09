//! Columnar storage - row to column transposition

use crate::error::Result;

/// A single column's data
#[derive(Debug, Clone)]
pub struct ColumnChunk {
    /// Column index
    pub column_index: usize,
    /// Column name
    pub column_name: String,
    /// Field type
    pub field_type: crate::schema::FieldType,
    /// Raw data bytes
    pub data: Vec<u8>,
    /// Number of rows
    pub row_count: u32,
}

impl ColumnChunk {
    /// Create new column chunk
    pub fn new(column_index: usize, column_name: String, field_type: crate::schema::FieldType) -> Self {
        ColumnChunk {
            column_index,
            column_name,
            field_type,
            data: Vec::new(),
            row_count: 0,
        }
    }

    /// Add bytes to column
    pub fn append(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Reset column
    pub fn reset(&mut self) {
        self.data.clear();
        self.row_count = 0;
    }

    /// Get column size in bytes
    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }
}

/// Row-oriented buffer for accumulating rows
#[derive(Debug)]
pub struct RowBuffer {
    /// Column count
    num_columns: usize,
    /// Rows stored as Vec of rows, each row is Vec of column data
    rows: Vec<Vec<Vec<u8>>>,
}

impl RowBuffer {
    /// Create new row buffer
    pub fn new(num_columns: usize) -> Self {
        RowBuffer {
            num_columns,
            rows: Vec::new(),
        }
    }

    /// Add a row
    pub fn add_row(&mut self, row: Vec<Vec<u8>>) -> Result<()> {
        if row.len() != self.num_columns {
            return Err(crate::error::Error::ColumnCountMismatch {
                expected: self.num_columns as u32,
                actual: row.len() as u32,
            });
        }

        self.rows.push(row);
        Ok(())
    }

    /// Get row count
    pub fn row_count(&self) -> u32 {
        self.rows.len() as u32
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Transpose to columnar format
    pub fn transpose(&self, field_types: &[crate::schema::FieldType]) -> Result<Vec<ColumnChunk>> {
        if self.rows.is_empty() {
            return Ok(Vec::new());
        }

        if field_types.len() != self.num_columns {
            return Err(crate::error::Error::ColumnCountMismatch {
                expected: self.num_columns as u32,
                actual: field_types.len() as u32,
            });
        }

        let mut columns: Vec<ColumnChunk> = (0..self.num_columns)
            .map(|i| ColumnChunk::new(i, format!("col_{}", i), field_types[i].clone()))
            .collect();

        for row in &self.rows {
            for (col_idx, col_data) in row.iter().enumerate() {
                columns[col_idx].append(col_data);
            }
        }

        for col in &mut columns {
            col.row_count = self.rows.len() as u32;
        }

        Ok(columns)
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.rows.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_chunk_creation() {
        let chunk = ColumnChunk::new(0, "id".to_string());
        assert_eq!(chunk.column_index, 0);
        assert_eq!(chunk.column_name, "id");
        assert_eq!(chunk.size_bytes(), 0);
    }

    #[test]
    fn test_row_buffer_creation() {
        let buffer = RowBuffer::new(3);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_row_buffer_add_row() {
        let mut buffer = RowBuffer::new(2);
        let row = vec![vec![1u8], vec![2u8]];
        assert!(buffer.add_row(row).is_ok());
        assert_eq!(buffer.row_count(), 1);
    }

    #[test]
    fn test_row_buffer_column_mismatch() {
        let mut buffer = RowBuffer::new(2);
        let row = vec![vec![1u8]]; // Only 1 column, but buffer expects 2
        assert!(buffer.add_row(row).is_err());
    }
}
