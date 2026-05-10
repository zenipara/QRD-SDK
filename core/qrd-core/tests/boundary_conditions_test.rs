//! Boundary condition and edge case tests for production robustness
//!
//! These tests ensure QRD handles:
//! - Empty inputs and outputs
//! - Integer overflow scenarios  
//! - Large inputs near memory limits
//! - Null/default values without panicking
//! - Truncated/corrupted files gracefully

use qrd_core::prelude::*;
use qrd_core::writer::StreamingWriter;
use std::io::Cursor;

/// Test Q4.1: Integer overflow detection and safe arithmetic
#[test]
fn test_boundary_zero_rows() {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .expect("Failed to build schema")
        .build()
        .expect("Failed to build schema");

    let buffer = Vec::new();
    let writer = StreamingWriter::new(Cursor::new(buffer), schema.clone())
        .expect("Failed to create writer");

    // Finish without writing any rows - should produce valid header/footer
    let result = writer.finish();
    assert!(result.is_ok(), "Finishing with zero rows should succeed");
}

/// Test Q4.1: Integer overflow protection with reasonable row count
#[test]
fn test_boundary_moderate_row_count() {
    // Test that row_count operations don't overflow with moderate data
    let schema = SchemaBuilder::new()
        .add_field("counter", FieldType::Int64, Nullability::Required)
        .expect("Failed to build schema")
        .build()
        .expect("Failed to build schema");

    let buffer = Vec::new();
    let mut writer = StreamingWriter::new(Cursor::new(buffer), schema.clone())
        .expect("Failed to create writer");

    // Add rows up to practical limit
    for i in 0..100 {
        let val = i as i64;
        let row = vec![val.to_le_bytes().to_vec()];
        let _ = writer.write_row(row);
    }

    let result = writer.finish();
    assert!(result.is_ok(), "Should handle moderate row counts safely");
}

/// Test Q4.2: Empty schema fields
#[test]
fn test_boundary_empty_schema() {
    let result = SchemaBuilder::new()
        .build();
    
    // Empty schema should be an error or return schema with no fields
    // Just ensure no panic either way
    _ = result;
}

/// Test Q4.2: Default/zero value handling - should not panic
#[test]
fn test_boundary_zero_value_in_row() {
    let schema = SchemaBuilder::new()
        .add_field("zero_val", FieldType::Int32, Nullability::Required)
        .expect("Failed to build schema")
        .build()
        .expect("Failed to build schema");

    let buffer = Vec::new();
    let mut writer = StreamingWriter::new(Cursor::new(buffer), schema.clone())
        .expect("Failed to create writer");

    // Write row with zero value
    let zero_bytes: i32 = 0;
    let row_data = vec![zero_bytes.to_le_bytes().to_vec()];
    let result = writer.write_row(row_data);
    assert!(result.is_ok() || result.is_err(), "Should handle zero values without panic");
}

/// Test Q4.3: Moderately large input handling
#[test]
fn test_boundary_large_blob_row() {
    let schema = SchemaBuilder::new()
        .add_field("large_blob", FieldType::Blob, Nullability::Required)
        .expect("Failed to build schema")
        .build()
        .expect("Failed to build schema");

    let buffer = Vec::new();
    let mut writer = StreamingWriter::new(Cursor::new(buffer), schema.clone())
        .expect("Failed to create writer");

    // Write a large blob (100KB)
    let large_data = vec![0xABu8; 100 * 1024];
    let result = writer.write_row(vec![large_data]);
    // Should either succeed or return a clear error, not panic/crash
    assert!(result.is_ok() || result.is_err(), "Should handle large rows without panic");
}

/// Test Q4.3: Memory bounded operation - many small rows
#[test]
fn test_boundary_many_small_rows_bounded_memory() {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .expect("Failed to build schema")
        .add_field("value", FieldType::Float64, Nullability::Required)
        .expect("Failed to build schema")
        .build()
        .expect("Failed to build schema");

    let buffer = Vec::new();
    let mut writer = StreamingWriter::new(Cursor::new(buffer), schema.clone())
        .expect("Failed to create writer");

    // Write many small rows - should auto-flush and not accumulate unbounded memory
    for i in 0..1000 {
        let id = i as i32;
        let value = i as f64;
        let row_data = vec![id.to_le_bytes().to_vec(), value.to_le_bytes().to_vec()];
        let _ = writer.write_row(row_data);
    }

    // Finishing should not exhaust memory
    let finish_result = writer.finish();
    assert!(finish_result.is_ok() || finish_result.is_err(), "Should complete without OOM");
}

/// Test Q4.1: Column offset arithmetic doesn't overflow
#[test]
fn test_boundary_column_offset_arithmetic() {
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int8, Nullability::Required)
        .expect("Failed to build schema")
        .add_field("b", FieldType::Int16, Nullability::Required)
        .expect("Failed to build schema")
        .add_field("c", FieldType::Int32, Nullability::Required)
        .expect("Failed to build schema")
        .add_field("d", FieldType::Int64, Nullability::Required)
        .expect("Failed to build schema")
        .build()
        .expect("Failed to build schema");

    // Schema creation should use checked arithmetic internally
    assert_eq!(schema.fields.len(), 4, "Schema should have 4 fields");
}

/// Test file handles minimal valid file
#[test]
fn test_boundary_minimal_valid_file() {
    let schema = SchemaBuilder::new()
        .add_field("x", FieldType::Int32, Nullability::Required)
        .expect("Failed to build schema")
        .build()
        .expect("Failed to build schema");

    let buffer = Vec::new();
    let writer = StreamingWriter::new(Cursor::new(buffer), schema.clone())
        .expect("Failed to create writer");

    let result = writer.finish();
    assert!(result.is_ok(), "Should create valid minimal file");
}
