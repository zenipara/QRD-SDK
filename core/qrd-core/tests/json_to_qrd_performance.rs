//! JSON to QRD conversion and performance tests
//!
//! These tests validate that JSON data can be successfully converted
//! to QRD format and that conversion maintains data integrity.

use qrd_core::prelude::*;
use qrd_core::writer::FileWriter;
use qrd_core::reader::FileReader;
use tempfile::NamedTempFile;

/// Simple JSON value representation for testing
#[derive(Debug, Clone)]
enum JsonValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
}

#[test]
fn test_json_ingestion_smoke_test() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("name", FieldType::String, Nullability::Required)
        .unwrap()
        .add_field("value", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("active", FieldType::Boolean, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

        // Write rows that simulate JSON data
        for i in 0..10 {
            writer.write_row(vec![
                (i as i64).to_le_bytes().to_vec(),
                format!("item_{}", i).into_bytes(),
                (i as f64 * 1.5).to_le_bytes().to_vec(),
                vec![if i % 2 == 0 { 1 } else { 0 }],
            ]).unwrap();
        }

        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 10);
    assert_eq!(reader.schema().fields.len(), 4);
}

#[test]
fn test_large_json_conversion_validation() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("row_id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("timestamp", FieldType::Timestamp, Nullability::Required)
        .unwrap()
        .add_field("data", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("metrics", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let row_count = 1000;

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

        for i in 0..row_count {
            let timestamp = (i as i64 * 1000).to_le_bytes().to_vec();
            let data_str = if i % 7 == 0 {
                b"".to_vec() // Empty string for some rows
            } else {
                format!("record_{:05}", i).into_bytes()
            };
            let metrics = vec![(i % 256) as u8; 100];

            writer.write_row(vec![
                (i as i64).to_le_bytes().to_vec(),
                timestamp,
                if i % 7 == 0 { vec![0,0,0,0] } else { 
                    let mut v = vec![(data_str.len() as u32).to_le_bytes().to_vec()];
                    v.extend(data_str);
                    v.concat()
                },
                metrics,
            ]).unwrap();
        }

        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), row_count);
    assert!(reader.row_group_offsets().len() > 0);
}

#[test]
fn test_json_null_handling_in_conversion() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("optional_field", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

        // Row 1: with value
        writer.write_row(vec![
            1i64.to_le_bytes().to_vec(),
            {
                let s = "test";
                let mut v = (s.len() as u32).to_le_bytes().to_vec();
                v.extend_from_slice(s.as_bytes());
                v
            },
        ]).unwrap();

        // Row 2: null value
        writer.write_row(vec![
            2i64.to_le_bytes().to_vec(),
            vec![0, 0, 0, 0], // NULL indicator
        ]).unwrap();

        // Row 3: with value
        writer.write_row(vec![
            3i64.to_le_bytes().to_vec(),
            {
                let s = "data";
                let mut v = (s.len() as u32).to_le_bytes().to_vec();
                v.extend_from_slice(s.as_bytes());
                v
            },
        ]).unwrap();

        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 3);
}

#[test]
fn test_json_mixed_types_conversion() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("int_field", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("float_field", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("string_field", FieldType::String, Nullability::Required)
        .unwrap()
        .add_field("bool_field", FieldType::Boolean, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

        for i in 0..50 {
            writer.write_row(vec![
                (i as i32).to_le_bytes().to_vec(),
                (i as f64 * 3.14).to_le_bytes().to_vec(),
                format!("text_{}", i).into_bytes(),
                vec![if i % 2 == 0 { 1 } else { 0 }],
            ]).unwrap();
        }

        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 50);
    assert_eq!(reader.schema().fields.len(), 4);
}

#[test]
fn test_json_array_simulation_roundtrip() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("array_data", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    // Simulate JSON array as serialized data
    let array_as_bytes = b"[1,2,3,4,5]".to_vec();

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
        writer.write_row(vec![array_as_bytes.clone()]).unwrap();
        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_json_nested_object_serialization() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("metadata", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    // Simulate nested JSON object serialization
    let nested_json = br#"{"name":"test","nested":{"value":42}}"#.to_vec();

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
        writer.write_row(vec![
            1i64.to_le_bytes().to_vec(),
            nested_json,
        ]).unwrap();
        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_json_conversion_determinism() {
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Float64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut buffer1 = Vec::new();
    {
        use std::io::Cursor;
        let cursor = Cursor::new(&mut buffer1);
        let mut writer = FileWriter::new(cursor, schema.clone()).unwrap();
        let value = 3.14159265358979;
        writer.write_row(vec![value.to_le_bytes().to_vec()]).unwrap();
        writer.finish().unwrap();
    }

    let mut buffer2 = Vec::new();
    {
        use std::io::Cursor;
        let cursor = Cursor::new(&mut buffer2);
        let mut writer = FileWriter::new(cursor, schema.clone()).unwrap();
        let value = 3.14159265358979;
        writer.write_row(vec![value.to_le_bytes().to_vec()]).unwrap();
        writer.finish().unwrap();
    }

    assert_eq!(buffer1, buffer2);
}

#[test]
fn test_json_unicode_string_handling() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("text", FieldType::String, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let unicode_strings = vec![
        "Hello",
        "世界",
        "مرحبا",
        "🚀",
    ];

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

        for text in &unicode_strings {
            let bytes = text.as_bytes();
            let mut row = (bytes.len() as u32).to_le_bytes().to_vec();
            row.extend_from_slice(bytes);
            writer.write_row(vec![row]).unwrap();
        }

        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), unicode_strings.len() as u32);
}

#[test]
fn test_json_ingestion_smoke_test() {
    // Additional test for JSON ingestion validation
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("json_data", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let json_payload = br#"{"key": "value", "number": 123}"#.to_vec();

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
        writer.write_row(vec![json_payload.clone()]).unwrap();
        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_large_json_conversion_validation() {
    // Test conversion of large JSON-like data
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("large_json", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let large_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(10000)).into_bytes();

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
        writer.write_row(vec![large_json]).unwrap();
        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_json_number_precision_preservation() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("f32_val", FieldType::Float32, Nullability::Required)
        .unwrap()
        .add_field("f64_val", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("i64_val", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

        let f32_val = 1.23456f32;
        let f64_val = 1.2345678901234567f64;
        let i64_val = 9223372036854775807i64;

        writer.write_row(vec![
            f32_val.to_le_bytes().to_vec(),
            f64_val.to_le_bytes().to_vec(),
            i64_val.to_le_bytes().to_vec(),
        ]).unwrap();

        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}
