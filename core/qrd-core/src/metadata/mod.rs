//! Metadata structures and utilities

use serde::{Deserialize, Serialize};

/// Column metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    /// Column name
    pub name: String,
    /// Compression codec ID
    pub compression_codec: u8,
    /// Encoding ID
    pub encoding: u8,
    /// Statistics for column
    pub statistics: Option<ColumnStatistics>,
}

/// Column statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStatistics {
    /// Minimum value (if applicable)
    pub min: Option<Vec<u8>>,
    /// Maximum value (if applicable)
    pub max: Option<Vec<u8>>,
    /// Null count
    pub null_count: u32,
    /// Distinct value count
    pub distinct_count: u32,
}

/// Row group metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowGroupMetadata {
    /// Number of rows in this group
    pub row_count: u32,
    /// Total size of uncompressed data
    pub total_uncompressed_size: u32,
    /// Total size of compressed data
    pub total_compressed_size: u32,
    /// Column offsets within the row group
    pub column_offsets: Vec<u64>,
}
