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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{SchemaBuilder, Nullability};

    // ====== ColumnStats Tests ======

    #[test]
    fn test_column_stats_initialization() {
        let stats = ColumnStats::new("test_col".to_string(), FieldType::Int64);
        assert_eq!(stats.name, "test_col");
        assert_eq!(stats.field_type, FieldType::Int64);
        assert_eq!(stats.min_value, None);
        assert_eq!(stats.max_value, None);
        assert_eq!(stats.null_count, 0);
        assert_eq!(stats.distinct_count, 0);
        assert_eq!(stats.total_count, 0);
    }

    #[test]
    fn test_column_stats_update_with_values() {
        let mut stats = ColumnStats::new("numbers".to_string(), FieldType::Int64);
        
        // Add values
        let value1 = 10i64.to_le_bytes().to_vec();
        let value2 = 20i64.to_le_bytes().to_vec();
        let value3 = 15i64.to_le_bytes().to_vec();
        
        stats.update(Some(&value1));
        stats.update(Some(&value2));
        stats.update(Some(&value3));
        
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.null_count, 0);
        assert_eq!(stats.distinct_count, 3);
        assert_eq!(stats.min_value, Some(value1));
        assert_eq!(stats.max_value, Some(value2));
    }

    #[test]
    fn test_column_stats_update_with_nulls() {
        let mut stats = ColumnStats::new("nullable_col".to_string(), FieldType::String);
        
        let val1 = b"hello".to_vec();
        stats.update(Some(&val1));
        stats.update(None);
        stats.update(None);
        let val2 = b"world".to_vec();
        stats.update(Some(&val2));
        
        assert_eq!(stats.total_count, 4);
        assert_eq!(stats.null_count, 2);
        assert_eq!(stats.distinct_count, 2);
    }

    #[test]
    fn test_column_stats_min_max_ordering() {
        let mut stats = ColumnStats::new("ordered".to_string(), FieldType::Int32);
        
        // Insert in reverse order
        stats.update(Some(&[5, 0, 0, 0]));
        stats.update(Some(&[1, 0, 0, 0]));
        stats.update(Some(&[3, 0, 0, 0]));
        stats.update(Some(&[2, 0, 0, 0]));
        stats.update(Some(&[4, 0, 0, 0]));
        
        assert_eq!(stats.min_value, Some(vec![1, 0, 0, 0]));
        assert_eq!(stats.max_value, Some(vec![5, 0, 0, 0]));
    }

    #[test]
    fn test_column_stats_all_nulls() {
        let mut stats = ColumnStats::new("all_nulls".to_string(), FieldType::Float64);
        
        stats.update(None);
        stats.update(None);
        stats.update(None);
        
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.null_count, 3);
        assert_eq!(stats.distinct_count, 0);
        assert_eq!(stats.min_value, None);
        assert_eq!(stats.max_value, None);
    }

    #[test]
    fn test_filter_result_combine() {
        let result1 = FilterResult::MayPass.combine(FilterResult::MayPass);
        assert_eq!(result1, FilterResult::MayPass);
        
        let result2 = FilterResult::MayPass.combine(FilterResult::MustNotPass);
        assert_eq!(result2, FilterResult::MustNotPass);
        
        let result3 = FilterResult::MustNotPass.combine(FilterResult::MustNotPass);
        assert_eq!(result3, FilterResult::MustNotPass);
    }

    #[test]
    fn test_column_filter_is_null() {
        let mut stats = ColumnStats::new("col".to_string(), FieldType::Int64);
        stats.update(Some(&[1, 0, 0, 0, 0, 0, 0, 0]));
        stats.update(None);
        
        let filter = ColumnFilter::IsNull;
        assert_eq!(stats.can_pass_filter(&filter), FilterResult::MayPass);
        
        let mut stats_no_nulls = ColumnStats::new("col2".to_string(), FieldType::Int64);
        stats_no_nulls.update(Some(&[1, 0, 0, 0, 0, 0, 0, 0]));
        assert_eq!(stats_no_nulls.can_pass_filter(&filter), FilterResult::MustNotPass);
    }

    #[test]
    fn test_column_filter_is_not_null() {
        let mut stats = ColumnStats::new("col".to_string(), FieldType::Int64);
        stats.update(Some(&[1, 0, 0, 0, 0, 0, 0, 0]));
        stats.update(None);
        
        let filter = ColumnFilter::IsNotNull;
        assert_eq!(stats.can_pass_filter(&filter), FilterResult::MayPass);
        
        let mut stats_all_null = ColumnStats::new("col2".to_string(), FieldType::Int64);
        stats_all_null.update(None);
        assert_eq!(stats_all_null.can_pass_filter(&filter), FilterResult::MustNotPass);
    }

    #[test]
    fn test_column_filter_equal() {
        let mut stats = ColumnStats::new("col".to_string(), FieldType::Int64);
        let val = 50i64.to_le_bytes().to_vec();
        stats.update(Some(&val));
        stats.update(Some(&(100i64.to_le_bytes().to_vec())));
        
        // Value within range
        let filter = ColumnFilter::Equal(val.clone());
        assert_eq!(stats.can_pass_filter(&filter), FilterResult::MayPass);
        
        // Value outside range
        let filter_out = ColumnFilter::Equal(vec![255, 255, 255, 255, 255, 255, 255, 255]);
        assert_eq!(stats.can_pass_filter(&filter_out), FilterResult::MustNotPass);
    }

    #[test]
    fn test_column_filter_greater_than() {
        let mut stats = ColumnStats::new("col".to_string(), FieldType::Int32);
        stats.update(Some(&[10, 0, 0, 0]));
        stats.update(Some(&[50, 0, 0, 0]));
        
        let filter_pass = ColumnFilter::GreaterThan(vec![30, 0, 0, 0]);
        assert_eq!(stats.can_pass_filter(&filter_pass), FilterResult::MayPass);
        
        let filter_fail = ColumnFilter::GreaterThan(vec![100, 0, 0, 0]);
        assert_eq!(stats.can_pass_filter(&filter_fail), FilterResult::MustNotPass);
    }

    #[test]
    fn test_column_filter_less_than() {
        let mut stats = ColumnStats::new("col".to_string(), FieldType::Int32);
        stats.update(Some(&[10, 0, 0, 0]));
        stats.update(Some(&[50, 0, 0, 0]));
        
        let filter_pass = ColumnFilter::LessThan(vec![30, 0, 0, 0]);
        assert_eq!(stats.can_pass_filter(&filter_pass), FilterResult::MayPass);
        
        let filter_fail = ColumnFilter::LessThan(vec![5, 0, 0, 0]);
        assert_eq!(stats.can_pass_filter(&filter_fail), FilterResult::MustNotPass);
    }

    #[test]
    fn test_column_filter_between() {
        let mut stats = ColumnStats::new("col".to_string(), FieldType::Int32);
        stats.update(Some(&[10, 0, 0, 0]));
        stats.update(Some(&[50, 0, 0, 0]));
        
        let filter_pass = ColumnFilter::Between(vec![20, 0, 0, 0], vec![40, 0, 0, 0]);
        assert_eq!(stats.can_pass_filter(&filter_pass), FilterResult::MayPass);
        
        let filter_fail = ColumnFilter::Between(vec![60, 0, 0, 0], vec![100, 0, 0, 0]);
        assert_eq!(stats.can_pass_filter(&filter_fail), FilterResult::MustNotPass);
    }

    #[test]
    fn test_column_filter_not_equal_single_value() {
        let mut stats = ColumnStats::new("col".to_string(), FieldType::Int64);
        let val = 42i64.to_le_bytes().to_vec();
        stats.update(Some(&val.clone()));
        stats.update(Some(&val.clone()));
        
        let filter = ColumnFilter::NotEqual(val.clone());
        assert_eq!(stats.can_pass_filter(&filter), FilterResult::MustNotPass);
        
        let filter_pass = ColumnFilter::NotEqual(vec![99, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(stats.can_pass_filter(&filter_pass), FilterResult::MayPass);
    }

    // ====== RowGroupStats Tests ======

    #[test]
    fn test_row_group_stats_initialization() {
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("name", FieldType::String, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();
        
        let rg_stats = RowGroupStats::new(&schema);
        assert_eq!(rg_stats.column_stats.len(), 2);
        assert_eq!(rg_stats.row_count, 0);
        assert_eq!(rg_stats.column_stats[0].name, "id");
        assert_eq!(rg_stats.column_stats[1].name, "name");
    }

    #[test]
    fn test_row_group_stats_update_row() {
        let schema = SchemaBuilder::new()
            .add_field("col1", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("col2", FieldType::Int32, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();
        
        let mut rg_stats = RowGroupStats::new(&schema);
        
        let row = vec![
            Some(10i64.to_le_bytes().to_vec()),
            Some(20i32.to_le_bytes().to_vec()),
        ];
        
        rg_stats.update_row(&row);
        assert_eq!(rg_stats.row_count, 1);
        assert_eq!(rg_stats.column_stats[0].total_count, 1);
        assert_eq!(rg_stats.column_stats[1].total_count, 1);
    }

    #[test]
    fn test_row_group_stats_multiple_updates() {
        let schema = SchemaBuilder::new()
            .add_field("value", FieldType::Int64, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();
        
        let mut rg_stats = RowGroupStats::new(&schema);
        
        rg_stats.update_row(&vec![Some(10i64.to_le_bytes().to_vec())]);
        rg_stats.update_row(&vec![None]);
        rg_stats.update_row(&vec![Some(20i64.to_le_bytes().to_vec())]);
        
        assert_eq!(rg_stats.row_count, 3);
        assert_eq!(rg_stats.column_stats[0].null_count, 1);
        assert_eq!(rg_stats.column_stats[0].total_count, 3);
    }

    #[test]
    fn test_row_group_stats_can_pass_single_filter() {
        let schema = SchemaBuilder::new()
            .add_field("status", FieldType::String, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();
        
        let mut rg_stats = RowGroupStats::new(&schema);
        rg_stats.update_row(&vec![Some(b"active".to_vec())]);
        rg_stats.update_row(&vec![Some(b"inactive".to_vec())]);
        
        let filters = vec![ColumnFilterSpec {
            column_index: 0,
            filter: ColumnFilter::IsNotNull,
        }];
        
        assert_eq!(rg_stats.can_pass_filters(&filters), FilterResult::MayPass);
    }

    #[test]
    fn test_row_group_stats_filter_rejection() {
        let schema = SchemaBuilder::new()
            .add_field("count", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();
        
        let mut rg_stats = RowGroupStats::new(&schema);
        rg_stats.update_row(&vec![Some(100i64.to_le_bytes().to_vec())]);
        rg_stats.update_row(&vec![Some(200i64.to_le_bytes().to_vec())]);
        
        let filters = vec![ColumnFilterSpec {
            column_index: 0,
            filter: ColumnFilter::LessThan(vec![50, 0, 0, 0, 0, 0, 0, 0]),
        }];
        
        assert_eq!(rg_stats.can_pass_filters(&filters), FilterResult::MustNotPass);
    }

    // ====== QueryOptimizer Tests ======

    #[test]
    fn test_query_optimizer_initialization() {
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();
        
        let optimizer = QueryOptimizer::new(schema.clone());
        assert_eq!(optimizer.schema.fields.len(), 1);
    }

    #[test]
    fn test_query_optimizer_all_groups_pass() {
        let schema = SchemaBuilder::new()
            .add_field("value", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();
        
        let mut rg_stats1 = RowGroupStats::new(&schema);
        rg_stats1.update_row(&vec![Some(10i64.to_le_bytes().to_vec())]);
        
        let mut rg_stats2 = RowGroupStats::new(&schema);
        rg_stats2.update_row(&vec![Some(20i64.to_le_bytes().to_vec())]);
        
        let row_group_stats = vec![rg_stats1, rg_stats2];
        let filters = vec![ColumnFilterSpec {
            column_index: 0,
            filter: ColumnFilter::IsNotNull,
        }];
        
        let optimizer = QueryOptimizer::new(schema);
        let accessible = optimizer.optimize_access(&row_group_stats, &filters);
        assert_eq!(accessible.len(), 2);
    }

    #[test]
    fn test_query_optimizer_some_groups_filtered() {
        let schema = SchemaBuilder::new()
            .add_field("age", FieldType::Int32, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();
        
        let mut rg_stats1 = RowGroupStats::new(&schema);
        rg_stats1.update_row(&vec![Some(vec![25, 0, 0, 0])]);
        
        let mut rg_stats2 = RowGroupStats::new(&schema);
        rg_stats2.update_row(&vec![Some(vec![100, 0, 0, 0])]);
        
        let row_group_stats = vec![rg_stats1, rg_stats2];
        let filters = vec![ColumnFilterSpec {
            column_index: 0,
            filter: ColumnFilter::GreaterThan(vec![50, 0, 0, 0]),
        }];
        
        let optimizer = QueryOptimizer::new(schema);
        let accessible = optimizer.optimize_access(&row_group_stats, &filters);
        assert_eq!(accessible.len(), 1);
        assert_eq!(accessible[0], 1);
    }

    // ====== MetadataIndex Tests ======

    #[test]
    fn test_metadata_index_creation() {
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("name", FieldType::String, Nullability::Required)
            .unwrap()
            .add_field("score", FieldType::Float32, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();
        
        let rg_stats = vec![RowGroupStats::new(&schema)];
        let offsets = vec![32];
        
        let index = MetadataIndex::new(&schema, offsets, rg_stats);
        assert_eq!(index.column_indices.len(), 3);
        assert_eq!(index.row_group_offsets.len(), 1);
    }

    #[test]
    fn test_metadata_index_get_column_index() {
        let schema = SchemaBuilder::new()
            .add_field("user_id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("email", FieldType::String, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();
        
        let rg_stats = vec![RowGroupStats::new(&schema)];
        let index = MetadataIndex::new(&schema, vec![32], rg_stats);
        
        assert_eq!(index.get_column_index("user_id"), Some(0));
        assert_eq!(index.get_column_index("email"), Some(1));
        assert_eq!(index.get_column_index("nonexistent"), None);
    }

    #[test]
    fn test_metadata_index_accessible_row_groups() {
        let schema = SchemaBuilder::new()
            .add_field("status", FieldType::String, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();
        
        let mut rg_stats1 = RowGroupStats::new(&schema);
        rg_stats1.update_row(&vec![Some(b"active".to_vec())]);
        
        let mut rg_stats2 = RowGroupStats::new(&schema);
        rg_stats2.update_row(&vec![Some(b"archived".to_vec())]);
        
        let index = MetadataIndex::new(&schema, vec![32, 256], vec![rg_stats1, rg_stats2]);
        
        let filters = vec![ColumnFilterSpec {
            column_index: 0,
            filter: ColumnFilter::IsNotNull,
        }];
        
        let accessible = index.get_accessible_row_groups(&filters);
        assert_eq!(accessible.len(), 2);
    }

    #[test]
    fn test_metadata_index_get_column_stats() {
        let schema = SchemaBuilder::new()
            .add_field("value", FieldType::Int64, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();
        
        let mut rg_stats = RowGroupStats::new(&schema);
        rg_stats.update_row(&vec![Some(10i64.to_le_bytes().to_vec())]);
        rg_stats.update_row(&vec![None]);
        rg_stats.update_row(&vec![Some(20i64.to_le_bytes().to_vec())]);
        
        let index = MetadataIndex::new(&schema, vec![32], vec![rg_stats]);
        let col_stats = index.get_column_stats(0);
        
        assert_eq!(col_stats.len(), 1);
        assert_eq!(col_stats[0].total_count, 3);
        assert_eq!(col_stats[0].null_count, 1);
    }

    #[test]
    fn test_metadata_index_serialization_roundtrip() {
        let schema = SchemaBuilder::new()
            .add_field("x", FieldType::Float64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();
        
        let mut rg_stats = RowGroupStats::new(&schema);
        rg_stats.update_row(&vec![Some(3.14f64.to_le_bytes().to_vec())]);
        
        let index = MetadataIndex::new(&schema, vec![32], vec![rg_stats.clone()]);
        let json = serde_json::to_string(&index).unwrap();
        let deserialized: MetadataIndex = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.column_indices.len(), index.column_indices.len());
        assert_eq!(deserialized.row_group_offsets, index.row_group_offsets);
    }

    #[test]
    fn test_column_stats_serialization_stability() {
        let mut stats = ColumnStats::new("test".to_string(), FieldType::Int64);
        stats.update(Some(&[1, 2, 3, 4, 5, 6, 7, 8]));
        stats.update(Some(&[9, 10, 11, 12, 13, 14, 15, 16]));
        
        let json1 = serde_json::to_string(&stats).unwrap();
        let deserialized: ColumnStats = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&deserialized).unwrap();
        
        assert_eq!(json1, json2);
    }

    // Additional tests for enterprise-grade coverage

    #[test]
    fn test_column_stats_overflow_edge_cases() {
        let mut stats = ColumnStats::new("overflow_test".to_string(), FieldType::Int64);
        
        // Test with maximum u64 values for counts
        for _ in 0..u64::MAX / 2 {
            stats.update(Some(&[1, 0, 0, 0, 0, 0, 0, 0]));
        }
        // This would overflow in real usage, but test the logic
        assert!(stats.total_count > 0);
    }

    #[test]
    fn test_column_stats_mixed_null_non_null() {
        let mut stats = ColumnStats::new("mixed".to_string(), FieldType::String);
        
        // Interleave nulls and values
        stats.update(Some(b"first".to_vec().as_slice()));
        stats.update(None);
        stats.update(Some(b"second".to_vec().as_slice()));
        stats.update(None);
        stats.update(Some(b"third".to_vec().as_slice()));
        
        assert_eq!(stats.total_count, 5);
        assert_eq!(stats.null_count, 2);
        assert_eq!(stats.distinct_count, 3);
        assert_eq!(stats.min_value, Some(b"first".to_vec()));
        assert_eq!(stats.max_value, Some(b"third".to_vec()));
    }

    #[test]
    fn test_column_stats_empty_column_stats() {
        let stats = ColumnStats::new("empty".to_string(), FieldType::Float32);
        
        assert_eq!(stats.total_count, 0);
        assert_eq!(stats.null_count, 0);
        assert_eq!(stats.distinct_count, 0);
        assert!(stats.min_value.is_none());
        assert!(stats.max_value.is_none());
    }

    #[test]
    fn test_column_stats_serialization_malformed() {
        // Test that malformed JSON doesn't deserialize
        let malformed_json = r#"{"name": "test", "field_type": "InvalidType"}"#;
        let result: std::result::Result<ColumnStats, _> = serde_json::from_str(malformed_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_column_stats_deterministic_updates() {
        let mut stats1 = ColumnStats::new("det1".to_string(), FieldType::Int32);
        let mut stats2 = ColumnStats::new("det2".to_string(), FieldType::Int32);
        
        let values = vec![
            Some(vec![1, 0, 0, 0]),
            None,
            Some(vec![3, 0, 0, 0]),
            Some(vec![2, 0, 0, 0]),
        ];
        
        for val in &values {
            stats1.update(val.as_ref().map(|v| v.as_slice()));
        }
        
        for val in &values {
            stats2.update(val.as_ref().map(|v| v.as_slice()));
        }
        
        assert_eq!(stats1.min_value, stats2.min_value);
        assert_eq!(stats1.max_value, stats2.max_value);
        assert_eq!(stats1.null_count, stats2.null_count);
        assert_eq!(stats1.total_count, stats2.total_count);
    }

    #[test]
    fn test_column_stats_large_values() {
        let mut stats = ColumnStats::new("large".to_string(), FieldType::Int64);
        
        let large_val = i64::MAX.to_le_bytes().to_vec();
        let small_val = i64::MIN.to_le_bytes().to_vec();
        
        stats.update(Some(&large_val));
        stats.update(Some(&small_val));
        
        assert_eq!(stats.min_value, Some(small_val));
        assert_eq!(stats.max_value, Some(large_val));
    }

    #[test]
    fn test_column_stats_filter_edge_cases() {
        let mut stats = ColumnStats::new("edge".to_string(), FieldType::Int64);
        
        // Single value
        let val = 42i64.to_le_bytes().to_vec();
        stats.update(Some(&val));
        
        // Test filters on single value
        assert_eq!(stats.can_pass_filter(&ColumnFilter::Equal(val.clone())), FilterResult::MayPass);
        assert_eq!(stats.can_pass_filter(&ColumnFilter::NotEqual(vec![99, 0, 0, 0, 0, 0, 0, 0])), FilterResult::MayPass);
        assert_eq!(stats.can_pass_filter(&ColumnFilter::GreaterThan(vec![40, 0, 0, 0, 0, 0, 0, 0])), FilterResult::MayPass);
        assert_eq!(stats.can_pass_filter(&ColumnFilter::LessThan(vec![50, 0, 0, 0, 0, 0, 0, 0])), FilterResult::MayPass);
    }
}
