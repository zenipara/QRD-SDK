//! Comprehensive boundary condition tests
//!
//! Tests edge cases and boundary conditions:
//! - Zero rows, max columns, empty blobs
//! - Very large datasets
//! - Extreme field values
//! - Schema validation boundaries

use qrd_core::writer::FileWriter;
use qrd_core::reader::FileReader;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use tempfile::NamedTempFile;
use std::fs::File;

/// Test zero rows (empty file roundtrip)
#[test]
fn test_boundary_zero_rows() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("value", FieldType::Float64, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 0);
    assert_eq!(reader.rows().unwrap().len(), 0);
}

/// Test maximum row count scenario (within practical limits)
#[test]
fn test_boundary_many_rows() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();

    // Write 1000000 rows
    for i in 0..1_000_000 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1_000_000);
}

/// Test maximum columns scenario
#[test]
fn test_boundary_many_columns() {
    let mut builder = SchemaBuilder::new();
    
    // Create schema with 256 columns (reasonable max)
    for i in 0..256 {
        builder = builder
            .add_field(
                &format!("col_{:03}", i),
                match i % 7 {
                    0 => FieldType::Int32,
                    1 => FieldType::Int64,
                    2 => FieldType::Float32,
                    3 => FieldType::Float64,
                    4 => FieldType::Boolean,
                    5 => FieldType::String,
                    _ => FieldType::Blob,
                },
                if i % 3 == 0 {
                    Nullability::Optional
                } else {
                    Nullability::Required
                },
            )
            .unwrap();
    }
    
    let schema = builder.build().unwrap();
    let temp = NamedTempFile::new().unwrap();
    
    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    
    let mut row = Vec::new();
    for i in 0..256 {
        match i % 7 {
            0 => row.push((i as i32).to_le_bytes().to_vec()),
            1 => row.push((i as i64).to_le_bytes().to_vec()),
            2 => row.push((i as f32).to_le_bytes().to_vec()),
            3 => row.push((i as f64).to_le_bytes().to_vec()),
            4 => row.push(vec![if i % 2 == 0 { 1 } else { 0 }]),
            5 => row.push(format!("col{}", i).as_bytes().to_vec()),
            _ => row.push(vec![i as u8; 10]),
        }
    }
    
    writer.write_row(row).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

/// Test empty string field
#[test]
fn test_boundary_empty_string() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("text", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    
    // Write empty string
    writer.write_row(vec![
        1i64.to_le_bytes().to_vec(),
        b"".to_vec(),
    ]).unwrap();
    
    // Write real string
    writer.write_row(vec![
        2i64.to_le_bytes().to_vec(),
        b"test".to_vec(),
    ]).unwrap();
    
    // Write empty string again
    writer.write_row(vec![
        3i64.to_le_bytes().to_vec(),
        b"".to_vec(),
    ]).unwrap();

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 3);
}

/// Test zero-length blob
#[test]
fn test_boundary_empty_blob() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("data", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    
    for i in 0..100 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
            if i % 2 == 0 {
                vec![]  // Empty blob
            } else {
                vec![0xFF; i]  // Sized blob
            },
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 100);
}

/// Test minimum blob (1 byte)
#[test]
fn test_boundary_single_byte_blob() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("byte", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    
    for i in 0..256 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
            vec![i as u8],
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 256);
}

/// Test maximum integer values
#[test]
fn test_boundary_int_extremes() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("int32_min", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("int32_max", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("int64_min", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("int64_max", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    
    writer.write_row(vec![
        i32::MIN.to_le_bytes().to_vec(),
        i32::MAX.to_le_bytes().to_vec(),
        i64::MIN.to_le_bytes().to_vec(),
        i64::MAX.to_le_bytes().to_vec(),
    ]).unwrap();

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

/// Test floating point extremes
#[test]
fn test_boundary_float_extremes() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("f32_min_positive", FieldType::Float32, Nullability::Required)
        .unwrap()
        .add_field("f32_max", FieldType::Float32, Nullability::Required)
        .unwrap()
        .add_field("f32_inf", FieldType::Float32, Nullability::Required)
        .unwrap()
        .add_field("f32_neg_inf", FieldType::Float32, Nullability::Required)
        .unwrap()
        .add_field("f64_min_positive", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("f64_max", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("f64_inf", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("f64_neg_inf", FieldType::Float64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    
    writer.write_row(vec![
        f32::MIN_POSITIVE.to_le_bytes().to_vec(),
        f32::MAX.to_le_bytes().to_vec(),
        f32::INFINITY.to_le_bytes().to_vec(),
        f32::NEG_INFINITY.to_le_bytes().to_vec(),
        f64::MIN_POSITIVE.to_le_bytes().to_vec(),
        f64::MAX.to_le_bytes().to_vec(),
        f64::INFINITY.to_le_bytes().to_vec(),
        f64::NEG_INFINITY.to_le_bytes().to_vec(),
    ]).unwrap();

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

/// Test mixed required and optional fields
#[test]
fn test_boundary_mixed_nullability() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("req_int", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("opt_int", FieldType::Int64, Nullability::Optional)
        .unwrap()
        .add_field("req_str", FieldType::String, Nullability::Required)
        .unwrap()
        .add_field("opt_str", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    
    // All fields present
    writer.write_row(vec![
        1i64.to_le_bytes().to_vec(),
        2i64.to_le_bytes().to_vec(),
        b"req".to_vec(),
        b"opt".to_vec(),
    ]).unwrap();
    
    // Optional fields null
    writer.write_row(vec![
        3i64.to_le_bytes().to_vec(),
        vec![],  // NULL optional int
        b"req".to_vec(),
        vec![],  // NULL optional string
    ]).unwrap();

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 2);
}

/// Test all-null optional fields row
#[test]
fn test_boundary_all_nulls() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("opt1", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("opt2", FieldType::Float64, Nullability::Optional)
        .unwrap()
        .add_field("opt3", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    
    for i in 0..100 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
            vec![],  // NULL
            vec![],  // NULL
            vec![],  // NULL
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 100);
}

/// Test very long strings (boundary test)
#[test]
fn test_boundary_very_long_string() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("text", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    
    // Create strings of various lengths
    let sizes = vec![
        1,
        10,
        100,
        1000,
        10_000,
        100_000,
        1_000_000,  // 1MB string
    ];
    
    for (i, size) in sizes.iter().enumerate() {
        let string = vec![b'A'; *size];
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
            string,
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), sizes.len() as u32);
}

/// Test single column schema
#[test]
fn test_boundary_single_column_schema() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("only_field", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    
    for i in 0..10 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 10);
}

/// Test all field types in single row
#[test]
fn test_boundary_all_types_single_row() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("int32", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("int64", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("float32", FieldType::Float32, Nullability::Required)
        .unwrap()
        .add_field("float64", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("bool", FieldType::Boolean, Nullability::Required)
        .unwrap()
        .add_field("string", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("blob", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    
    writer.write_row(vec![
        (-42i32).to_le_bytes().to_vec(),
        (1234567890i64).to_le_bytes().to_vec(),
        (3.14f32).to_le_bytes().to_vec(),
        (2.718f64).to_le_bytes().to_vec(),
        vec![1],  // true
        b"test string".to_vec(),
        vec![0x00, 0x01, 0x02, 0xFF],
    ]).unwrap();

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

/// Test exactly at row group boundary
#[test]
fn test_boundary_row_group_boundary() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let config = qrd_core::writer::WriterConfig {
        row_group_size: 100,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    // Write exactly at row group boundaries
    for i in 0..300 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 300);
}

/// Test just below row group boundary
#[test]
fn test_boundary_row_group_below() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let config = qrd_core::writer::WriterConfig {
        row_group_size: 100,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    // Write just below row group boundary
    for i in 0..99 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 99);
}

/// Test just above row group boundary
#[test]
fn test_boundary_row_group_above() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let config = qrd_core::writer::WriterConfig {
        row_group_size: 100,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    // Write just above row group boundary
    for i in 0..101 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 101);
}
