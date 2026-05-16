//! Edge case tests for QRD Core
//! Tests boundary conditions, extreme values, and special data patterns

use qrd_core::compression::CompressionCodec;
use qrd_core::encoding::EncodingType;
use qrd_core::prelude::*;
use qrd_core::reader::FileReader;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::FileWriter;
use std::io::Cursor;
use tempfile::NamedTempFile;

/// Test empty file creation
#[test]
fn test_empty_file_roundtrip() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 0);
    assert_eq!(reader.rows().unwrap().len(), 0);
}

/// Test single row file
#[test]
fn test_single_row_file() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer
        .write_row(vec![(42i32).to_le_bytes().to_vec()])
        .unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

/// Test maximum i64 values
#[test]
fn test_max_min_int64_values() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer
        .write_row(vec![i64::MAX.to_le_bytes().to_vec()])
        .unwrap();
    writer
        .write_row(vec![i64::MIN.to_le_bytes().to_vec()])
        .unwrap();
    writer.write_row(vec![0i64.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 3);
}

/// Test empty string field
#[test]
fn test_empty_string_field() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("text", FieldType::String, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    // Empty string: just 4-byte length prefix with 0
    let empty_string = (0u32).to_le_bytes().to_vec();
    writer.write_row(vec![empty_string]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

/// Test very large string field
#[test]
fn test_large_string_field() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("text", FieldType::String, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    // Create 1MB string
    let large_text = "x".repeat(1024 * 1024);
    let mut text_bytes = Vec::new();
    text_bytes.extend_from_slice(&(large_text.len() as u32).to_le_bytes());
    text_bytes.extend_from_slice(large_text.as_bytes());

    writer.write_row(vec![text_bytes]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

/// Test NaN and infinity floats
#[test]
fn test_float_special_values() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Float64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer
        .write_row(vec![f64::NAN.to_le_bytes().to_vec()])
        .unwrap();
    writer
        .write_row(vec![f64::INFINITY.to_le_bytes().to_vec()])
        .unwrap();
    writer
        .write_row(vec![f64::NEG_INFINITY.to_le_bytes().to_vec()])
        .unwrap();
    writer
        .write_row(vec![0.0f64.to_le_bytes().to_vec()])
        .unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 4);
}

/// Test zero-size blob
#[test]
fn test_zero_size_blob() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("data", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    // Zero-size blob: just 4-byte length with 0
    let zero_blob = (0u32).to_le_bytes().to_vec();
    writer.write_row(vec![zero_blob]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

/// Test many columns with different types
#[test]
fn test_many_columns() {
    let temp = NamedTempFile::new().unwrap();
    let mut builder = SchemaBuilder::new();

    for i in 0..20 {
        builder = builder
            .add_field(
                &format!("col_{}", i),
                FieldType::Int32,
                Nullability::Required,
            )
            .unwrap();
    }

    let schema = builder.build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();

    let mut row = Vec::new();
    for i in 0..20 {
        row.push((i as i32).to_le_bytes().to_vec());
    }
    writer.write_row(row).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.schema().fields.len(), 20);
    assert_eq!(reader.row_count(), 1);
}

/// Test alternating boolean pattern
#[test]
fn test_alternating_booleans() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("flag", FieldType::Boolean, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..1000 {
        let flag = (i % 2 == 0) as u8;
        writer.write_row(vec![vec![flag]]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1000);
}

/// Test sequential incremental integers
#[test]
fn test_sequential_integers() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("seq", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..1000 {
        writer
            .write_row(vec![(i as i64).to_le_bytes().to_vec()])
            .unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1000);
}

/// Test highly repetitive data (good compression candidate)
#[test]
fn test_highly_repetitive_data() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("data", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    let pattern = b"AAAAAAAAAA";

    for _ in 0..100 {
        let mut data = vec![10u32.to_le_bytes().to_vec()];
        data[0].extend_from_slice(pattern);
        writer.write_row(data).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 100);
}

/// Test random-looking data (poor compression candidate)
#[test]
fn test_random_looking_data() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("data", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    let mut seed = 12345u64;

    for _ in 0..100 {
        seed = seed.wrapping_mul(2654435761).wrapping_add(2246822519);
        writer.write_row(vec![seed.to_le_bytes().to_vec()]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 100);
}

/// Test row count at power of 2 boundaries
#[test]
fn test_power_of_2_row_counts() {
    for power in 4..12 {
        let count = 1usize << power; // 2^power
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let mut writer = FileWriter::new(temp.path(), schema).unwrap();
        for i in 0..count {
            writer
                .write_row(vec![(i as i64).to_le_bytes().to_vec()])
                .unwrap();
        }
        writer.finish().unwrap();

        let reader = FileReader::new(temp.path()).unwrap();
        assert_eq!(
            reader.row_count() as usize,
            count,
            "Failed for 2^{} rows",
            power
        );
    }
}

/// Test all compression codecs with same data
#[test]
fn test_compression_codec_comparison() {
    let codecs = vec![
        CompressionCodec::None,
        CompressionCodec::Zstd,
        CompressionCodec::Lz4,
    ];

    for codec in codecs {
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("data", FieldType::Blob, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let test_data: Vec<u8> = (0..100).map(|i| (i % 256) as u8).collect();
        let mut writer = FileWriter::new(temp.path(), schema).unwrap();

        for _ in 0..10 {
            let mut row = vec![10u32.to_le_bytes().to_vec()];
            row[0].extend_from_slice(&test_data);
            writer.write_row(row).unwrap();
        }
        writer.finish().unwrap();

        let reader = FileReader::new(temp.path()).unwrap();
        assert_eq!(reader.row_count(), 10, "Failed for codec: {:?}", codec);
    }
}
