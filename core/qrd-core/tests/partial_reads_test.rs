//! Tests for v1.1.0 Partial Reads features
//!
//! Tests column statistics collection, query pushdown optimization,
//! and metadata indexing functionality.

use qrd_core::metadata::{ColumnFilter, ColumnFilterSpec, QueryOptimizer};
use qrd_core::prelude::*;
use qrd_core::reader::PartialReader;
use qrd_core::writer::{FileWriter, WriterConfig};
use tempfile::NamedTempFile;

#[test]
#[ignore]
fn test_column_statistics_collection() {
    let temp = NamedTempFile::new().unwrap();

    // Create schema
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("score", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("category", FieldType::String, Nullability::Required)
        .unwrap()
        .add_field("optional_field", FieldType::Int32, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    // Write test data
    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

        // Write rows with known statistics
        for i in 0..100 {
            let id_bytes = (i as i64).to_le_bytes().to_vec();
            let score_bytes = ((i as f64) * 1.5).to_le_bytes().to_vec();
            let category = if i % 3 == 0 {
                "A"
            } else if i % 3 == 1 {
                "B"
            } else {
                "C"
            };
            let category_bytes = serialize_string(category);

            // Optional field: null for even i, value for odd i
            let optional_bytes = if i % 2 == 0 {
                vec![] // Empty = null
            } else {
                (i as i32).to_le_bytes().to_vec()
            };

            writer
                .write_row(vec![id_bytes, score_bytes, category_bytes, optional_bytes])
                .unwrap();
        }

        writer.finish().unwrap();
    }

    // Read and verify statistics
    let mut reader = PartialReader::new(
        std::fs::File::open(temp.path()).unwrap(),
        Default::default(),
    )
    .unwrap();

    // Check that metadata index exists
    let metadata_index = reader.metadata_index().unwrap();

    // Verify column statistics for 'id' column (sequential 0-99)
    let id_stats = reader.get_column_statistics("id").unwrap();
    assert_eq!(id_stats.len(), 1); // One row group
    let rg_stats = &id_stats[0];
    assert_eq!(rg_stats.total_count, 100);
    assert_eq!(rg_stats.null_count, 0);
    assert!(rg_stats.min_value.is_some());
    assert!(rg_stats.max_value.is_some());

    // Verify min/max values for id
    let min_id = i64::from_le_bytes(
        rg_stats.min_value.as_ref().unwrap()[..8]
            .try_into()
            .unwrap(),
    );
    let max_id = i64::from_le_bytes(
        rg_stats.max_value.as_ref().unwrap()[..8]
            .try_into()
            .unwrap(),
    );
    assert_eq!(min_id, 0);
    assert_eq!(max_id, 99);

    // Verify optional field statistics
    let optional_stats = reader.get_column_statistics("optional_field").unwrap();
    let opt_rg_stats = &optional_stats[0];
    assert_eq!(opt_rg_stats.total_count, 100);
    assert_eq!(opt_rg_stats.null_count, 50); // Half are null
}

#[test]
#[ignore]
fn test_query_pushdown_optimization() {
    let temp = NamedTempFile::new().unwrap();

    // Create schema
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("status", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    // Write test data with multiple row groups
    {
        let mut config = WriterConfig::default();
        config.row_group_size = 50;
        config.compression_level = 1;

        let mut writer = FileWriter::with_config(
            std::fs::File::create(temp.path()).unwrap(),
            schema.clone(),
            config,
        )
        .unwrap();

        for i in 0..200 {
            let id_bytes = (i as i64).to_le_bytes().to_vec();
            let status = if i < 100 { 1i32 } else { 2i32 };
            let status_bytes = status.to_le_bytes().to_vec();

            writer.write_row(vec![id_bytes, status_bytes]).unwrap();
        }

        writer.finish().unwrap();
    }

    // Test query pushdown
    let mut reader = PartialReader::new(
        std::fs::File::open(temp.path()).unwrap(),
        Default::default(),
    )
    .unwrap();

    // Filter for status = 1 (should only match first row group)
    let filters = vec![ColumnFilterSpec {
        column_index: 1, // status column
        filter: ColumnFilter::Equal(1i32.to_le_bytes().to_vec()),
    }];

    // Estimate result count
    let estimated_count = reader.estimate_query_result_count(&filters);
    assert!(estimated_count > 0);

    // Read with filters
    let result = reader.read_columns_with_filters(&[0, 1], &filters).unwrap();

    // Should only return rows from first row group (ids 0-49)
    assert!(!result.is_empty());
    // Verify all returned rows have status = 1
    for row in result.chunks(2) {
        // 2 columns per row
        if let Some(status_bytes) = row.get(1) {
            let status = i32::from_le_bytes(status_bytes[..4].try_into().unwrap());
            assert_eq!(status, 1);
        }
    }
}

#[test]
#[ignore]
fn test_column_selective_reads() {
    let temp = NamedTempFile::new().unwrap();

    // Create schema
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("name", FieldType::String, Nullability::Required)
        .unwrap()
        .add_field("score", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("active", FieldType::Boolean, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    // Write test data
    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

        for i in 0..10 {
            let id_bytes = (i as i64).to_le_bytes().to_vec();
            let name_bytes = serialize_string(&format!("user_{}", i));
            let score_bytes = (i as f64 * 10.0).to_le_bytes().to_vec();
            let active_bytes = vec![(i % 2 == 0) as u8];

            writer
                .write_row(vec![id_bytes, name_bytes, score_bytes, active_bytes])
                .unwrap();
        }

        writer.finish().unwrap();
    }

    // Test column-selective reads
    let mut reader = PartialReader::new(
        std::fs::File::open(temp.path()).unwrap(),
        Default::default(),
    )
    .unwrap();

    // Read only id and score columns (indices 0 and 2)
    let column_indices = vec![0, 2];
    let result = reader
        .read_columns_with_filters(&column_indices, &[])
        .unwrap();

    // Verify we got data
    assert!(!result.is_empty());

    // Test reading by column names
    let result_by_name = reader.read_columns_by_name(&["id", "score"], &[]).unwrap();
    assert_eq!(result, result_by_name);
}

#[test]
#[ignore]
fn test_metadata_index_functionality() {
    let temp = NamedTempFile::new().unwrap();

    // Create schema
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("value", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    // Write data
    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
        for i in 0..50 {
            let id_bytes = (i as i64).to_le_bytes().to_vec();
            let value_bytes = (i as i32 * 2).to_le_bytes().to_vec();
            writer.write_row(vec![id_bytes, value_bytes]).unwrap();
        }
        writer.finish().unwrap();
    }

    // Test metadata index
    let mut reader = PartialReader::new(
        std::fs::File::open(temp.path()).unwrap(),
        Default::default(),
    )
    .unwrap();
    let metadata_index = reader.metadata_index().unwrap();

    // Test column index lookup
    assert_eq!(metadata_index.get_column_index("id"), Some(0));
    assert_eq!(metadata_index.get_column_index("value"), Some(1));
    assert_eq!(metadata_index.get_column_index("nonexistent"), None);

    // Test row group access
    assert_eq!(metadata_index.row_group_offsets.len(), 1); // One row group
    assert_eq!(metadata_index.row_group_stats.len(), 1);

    // Test column statistics access
    let id_stats = metadata_index.get_column_stats(0);
    assert_eq!(id_stats.len(), 1);
    assert_eq!(id_stats[0].total_count, 50);
}

#[test]
fn test_query_optimizer() {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("status", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let optimizer = QueryOptimizer::new(schema.clone());

    // Create mock row group stats
    let mut rg_stats = vec![
        qrd_core::metadata::RowGroupStats::new(&schema),
        qrd_core::metadata::RowGroupStats::new(&schema),
    ];

    // Simulate data: first RG has status=1, second RG has status=2
    for i in 0..10 {
        let status_val = if i < 5 { 1i32 } else { 2i32 };
        let status_bytes = status_val.to_le_bytes().to_vec();
        rg_stats[0].update_row(&[Some((i as i64).to_le_bytes().to_vec()), Some(status_bytes)]);
    }

    for i in 10..20 {
        let status_val = if i < 15 { 1i32 } else { 2i32 };
        let status_bytes = status_val.to_le_bytes().to_vec();
        rg_stats[1].update_row(&[Some((i as i64).to_le_bytes().to_vec()), Some(status_bytes)]);
    }

    // Test filter optimization
    let filters = vec![ColumnFilterSpec {
        column_index: 1, // status column
        filter: ColumnFilter::Equal(1i32.to_le_bytes().to_vec()),
    }];

    let accessible_groups = optimizer.optimize_access(&rg_stats, &filters);
    assert!(!accessible_groups.is_empty()); // Should find some groups

    let estimate = optimizer.estimate_result_count(&rg_stats, &filters);
    assert!(estimate > 0);
}

/// Helper function to serialize strings
fn serialize_string(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let len = bytes.len() as u32;
    result.extend_from_slice(&len.to_le_bytes());
    result.extend_from_slice(bytes);
    result
}
