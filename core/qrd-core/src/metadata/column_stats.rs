//! Column statistics collection and analysis
//!
//! Provides:
//! - StatisticsCollector: Collects min/max/null/distinct counts during writing
//! - Query pushdown: Filter evaluation using column statistics
//! - Metadata indexing: Efficient column lookup and access

use crate::schema::{FieldType, Schema};
use serde::{Deserialize, Serialize};

/// Column statistics for a single column
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnStats {
    /// Column name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Minimum value (serialized bytes)
    pub min_value: Option<Vec<u8>>,
    /// Maximum value (serialized bytes)
    pub max_value: Option<Vec<u8>>,
    /// Number of null values
    pub null_count: u64,
    /// Number of distinct values (approximate)
    pub distinct_count: u64,
    /// Total number of values
    pub total_count: u64,
}

impl ColumnStats {
    /// Create new column statistics
    pub fn new(name: String, field_type: FieldType) -> Self {
        ColumnStats {
            name,
            field_type,
            min_value: None,
            max_value: None,
            null_count: 0,
            distinct_count: 0,
            total_count: 0,
        }
    }

    /// Update statistics with a new value
    pub fn update(&mut self, value: Option<&[u8]>) {
        self.total_count += 1;

        match value {
            Some(val) => {
                // Update min/max using safe pattern matching instead of unwrap()
                match &self.min_value {
                    None => self.min_value = Some(val.to_vec()),
                    Some(min) if val < min => self.min_value = Some(val.to_vec()),
                    _ => {}
                }
                match &self.max_value {
                    None => self.max_value = Some(val.to_vec()),
                    Some(max) if val > max => self.max_value = Some(val.to_vec()),
                    _ => {}
                }

                // Update distinct count (simple approximation)
                // In production, you'd use HyperLogLog or similar
                self.distinct_count += 1;
            }
            None => {
                self.null_count += 1;
            }
        }
    }

    /// Check if a value would pass a filter based on statistics
    pub fn can_pass_filter(&self, filter: &ColumnFilter) -> FilterResult {
        match filter {
            ColumnFilter::IsNull => {
                if self.null_count > 0 {
                    FilterResult::MayPass
                } else {
                    FilterResult::MustNotPass
                }
            }
            ColumnFilter::IsNotNull => {
                if self.total_count > self.null_count {
                    FilterResult::MayPass
                } else {
                    FilterResult::MustNotPass
                }
            }
            ColumnFilter::Equal(value) => {
                // Check if value is within min/max range
                if let (Some(min), Some(max)) = (&self.min_value, &self.max_value) {
                    if value >= min && value <= max {
                        FilterResult::MayPass
                    } else {
                        FilterResult::MustNotPass
                    }
                } else {
                    FilterResult::MayPass
                }
            }
            ColumnFilter::NotEqual(value) => {
                // If all values are the same and equal to the filter value, must not pass
                if self.distinct_count == 1 && self.min_value.as_ref() == Some(value) {
                    FilterResult::MustNotPass
                } else {
                    FilterResult::MayPass
                }
            }
            ColumnFilter::GreaterThan(value) => {
                if let Some(max) = &self.max_value {
                    if max > value {
                        FilterResult::MayPass
                    } else {
                        FilterResult::MustNotPass
                    }
                } else {
                    FilterResult::MayPass
                }
            }
            ColumnFilter::LessThan(value) => {
                if let Some(min) = &self.min_value {
                    if min < value {
                        FilterResult::MayPass
                    } else {
                        FilterResult::MustNotPass
                    }
                } else {
                    FilterResult::MayPass
                }
            }
            ColumnFilter::Between(min_val, max_val) => {
                if let (Some(col_min), Some(col_max)) = (&self.min_value, &self.max_value) {
                    // Check if ranges overlap
                    if col_max >= min_val && col_min <= max_val {
                        FilterResult::MayPass
                    } else {
                        FilterResult::MustNotPass
                    }
                } else {
                    FilterResult::MayPass
                }
            }
        }
    }
}

/// Statistics collector for all columns in a row group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowGroupStats {
    /// Statistics for each column
    pub column_stats: Vec<ColumnStats>,
    /// Row count in this group
    pub row_count: u64,
}

impl RowGroupStats {
    /// Create new row group statistics
    pub fn new(schema: &Schema) -> Self {
        let column_stats = schema
            .fields
            .iter()
            .map(|field| ColumnStats::new(field.name.clone(), field.field_type))
            .collect();

        RowGroupStats {
            column_stats,
            row_count: 0,
        }
    }

    /// Update statistics with a row
    pub fn update_row(&mut self, row: &[Option<Vec<u8>>]) {
        self.row_count += 1;

        for (col_idx, value) in row.iter().enumerate() {
            if col_idx < self.column_stats.len() {
                self.column_stats[col_idx].update(value.as_ref().map(|v| v.as_slice()));
            }
        }
    }

    /// Check if row group can pass a set of filters
    pub fn can_pass_filters(&self, filters: &[ColumnFilterSpec]) -> FilterResult {
        let mut result = FilterResult::MayPass;

        for filter_spec in filters {
            if let Some(col_stats) = self.column_stats.get(filter_spec.column_index) {
                let col_result = col_stats.can_pass_filter(&filter_spec.filter);
                result = result.combine(col_result);

                // Early exit if we know it must not pass
                if result == FilterResult::MustNotPass {
                    break;
                }
            }
        }

        result
    }
}

/// Column filter for query pushdown
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColumnFilter {
    /// Value must be null
    IsNull,
    /// Value must not be null
    IsNotNull,
    /// Value must equal the specified bytes
    Equal(Vec<u8>),
    /// Value must not equal the specified bytes
    NotEqual(Vec<u8>),
    /// Value must be greater than the specified bytes
    GreaterThan(Vec<u8>),
    /// Value must be less than the specified bytes
    LessThan(Vec<u8>),
    /// Value must be between min and max (inclusive)
    Between(Vec<u8>, Vec<u8>),
}

/// Filter specification with column index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnFilterSpec {
    /// Column index to filter
    pub column_index: usize,
    /// Filter to apply
    pub filter: ColumnFilter,
}

/// Result of filter evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterResult {
    /// Row group may contain matching rows
    MayPass,
    /// Row group definitely does not contain matching rows
    MustNotPass,
}

impl FilterResult {
    /// Combine two filter results
    pub fn combine(self, other: FilterResult) -> FilterResult {
        match (self, other) {
            (FilterResult::MustNotPass, _) | (_, FilterResult::MustNotPass) => {
                FilterResult::MustNotPass
            }
            _ => FilterResult::MayPass,
        }
    }
}

/// Query pushdown optimizer
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryOptimizer {
    /// Schema for the query
    schema: Schema,
}

impl QueryOptimizer {
    /// Create new query optimizer
    pub fn new(schema: Schema) -> Self {
        QueryOptimizer { schema }
    }

    /// Optimize row group access based on filters
    pub fn optimize_access(
        &self,
        row_group_stats: &[RowGroupStats],
        filters: &[ColumnFilterSpec],
    ) -> Vec<usize> {
        let mut accessible_groups = Vec::new();

        for (idx, stats) in row_group_stats.iter().enumerate() {
            if stats.can_pass_filters(filters) != FilterResult::MustNotPass {
                accessible_groups.push(idx);
            }
        }

        accessible_groups
    }

    /// Estimate result count based on statistics
    pub fn estimate_result_count(
        &self,
        row_group_stats: &[RowGroupStats],
        filters: &[ColumnFilterSpec],
    ) -> u64 {
        let mut total_estimate = 0u64;

        for stats in row_group_stats {
            match stats.can_pass_filters(filters) {
                FilterResult::MayPass => {
                    // Simple estimation: assume 50% selectivity for may-pass groups
                    total_estimate += stats.row_count / 2;
                }
                FilterResult::MustNotPass => {
                    // Skip this group
                }
            }
        }

        total_estimate
    }
}

/// Metadata index for efficient column access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataIndex {
    /// Column name to index mapping
    pub column_indices: std::collections::HashMap<String, usize>,
    /// Row group offsets
    pub row_group_offsets: Vec<u64>,
    /// Row group statistics
    pub row_group_stats: Vec<RowGroupStats>,
}

impl MetadataIndex {
    /// Create metadata index from footer and statistics
    pub fn new(
        schema: &Schema,
        row_group_offsets: Vec<u64>,
        row_group_stats: Vec<RowGroupStats>,
    ) -> Self {
        let mut column_indices = std::collections::HashMap::new();

        for (idx, field) in schema.fields.iter().enumerate() {
            column_indices.insert(field.name.clone(), idx);
        }

        MetadataIndex {
            column_indices,
            row_group_offsets,
            row_group_stats,
        }
    }

    /// Get column index by name
    pub fn get_column_index(&self, column_name: &str) -> Option<usize> {
        self.column_indices.get(column_name).copied()
    }

    /// Get row groups that can satisfy filters
    pub fn get_accessible_row_groups(&self, filters: &[ColumnFilterSpec]) -> Vec<usize> {
        let mut accessible_groups = Vec::new();

        for (idx, stats) in self.row_group_stats.iter().enumerate() {
            if stats.can_pass_filters(filters) != FilterResult::MustNotPass {
                accessible_groups.push(idx);
            }
        }

        accessible_groups
    }

    /// Get statistics for a specific column across all row groups
    pub fn get_column_stats(&self, column_index: usize) -> Vec<&ColumnStats> {
        self.row_group_stats
            .iter()
            .filter_map(|rg_stats| rg_stats.column_stats.get(column_index))
            .collect()
    }
}
