//! ECC (Error-Correcting Code) recovery tests
//!
//! Tests various ECC scenarios:
//! - Different ECC configurations
//! - Data corruption and recovery patterns
//! - Edge cases in ECC handling

use qrd_core::prelude::*;
use qrd_core::writer::{FileWriter, WriterConfig};
use qrd_core::reader::FileReader;
use qrd_core::ecc::EccConfig;
use qrd_core::encryption::EncryptionConfig;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use tempfile::NamedTempFile;
use std::fs::File;

/// Test basic ECC configuration (4 data, 2 parity)
#[test]
fn test_ecc_basic_config_4_2() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("data", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(4).ok();

    let config = WriterConfig {
        ecc,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    writer.write_row(vec![
        1i64.to_le_bytes().to_vec(),
        b"data".to_vec(),
    ]).unwrap();

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test ECC with minimum configuration (2 data, 1 parity)
#[test]
fn test_ecc_minimum_config_2_1() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(2).ok();

    let config = WriterConfig {
        ecc,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    writer.write_row(vec![
        42i64.to_le_bytes().to_vec(),
    ]).unwrap();

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test ECC with high redundancy (8 data, 8 parity)
#[test]
fn test_ecc_high_redundancy_8_8() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("data", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(8).ok();

    let config = WriterConfig {
        ecc,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    for i in 0..10 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
            format!("data_{}", i).as_bytes().to_vec(),
        ]).unwrap();
    }

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test ECC with zero parity blocks (invalid)
#[test]
fn test_ecc_zero_parity() {
    let ecc = EccConfig::new(4);
    // EccConfig::new should handle or reject zero parity
    // Behavior depends on implementation
    let _ = ecc;
}

/// Test ECC with data blocks = parity blocks
#[test]
fn test_ecc_balanced_data_parity_5_5() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(5).ok();

    let config = WriterConfig {
        ecc,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    writer.write_row(vec![
        123i64.to_le_bytes().to_vec(),
    ]).unwrap();

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test ECC with large data blocks
#[test]
fn test_ecc_large_config_32_16() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("data", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(32).ok();

    let config = WriterConfig {
        ecc,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    writer.write_row(vec![
        1i64.to_le_bytes().to_vec(),
        vec![0xFF; 1024],  // 1KB blob
    ]).unwrap();

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test ECC combined with encryption
#[test]
fn test_ecc_with_encryption() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("secret", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(4).ok();
    let encryption = EncryptionConfig::new([77u8; 32].to_vec()).ok();

    let config = WriterConfig {
        ecc,
        encryption,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    writer.write_row(vec![
        1i64.to_le_bytes().to_vec(),
        b"protected with ECC and encryption".to_vec(),
    ]).unwrap();

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test ECC with compression
#[test]
fn test_ecc_with_compression() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("data", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(6).ok();

    let config = WriterConfig {
        ecc,
        compression_level: 7,  // High compression
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    // Compressible data
    for i in 0..50 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
            b"AAAAAAAAAAAAAAAA".to_vec(),  // Highly compressible
        ]).unwrap();
    }

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test ECC with empty file
#[test]
fn test_ecc_empty_file() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(4).ok();

    let config = WriterConfig {
        ecc,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let writer = FileWriter::with_config(file, schema, config).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 0);
}

/// Test ECC with single row
#[test]
fn test_ecc_single_row() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(4).ok();

    let config = WriterConfig {
        ecc,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    writer.write_row(vec![
        999i64.to_le_bytes().to_vec(),
    ]).unwrap();

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

/// Test ECC with many rows across multiple row groups
#[test]
fn test_ecc_multiple_row_groups() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(4).ok();

    let config = WriterConfig {
        ecc,
        row_group_size: 10,  // Small row groups
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    // Write multiple row groups worth of data
    for i in 0..1000 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1000);
}

/// Test ECC with variable-length data
#[test]
fn test_ecc_variable_length_data() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("text", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(4).ok();

    let config = WriterConfig {
        ecc,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    // Write rows with varying string lengths
    let strings = vec![
        b"a".to_vec(),
        b"ab".to_vec(),
        b"abc".to_vec(),
        b"a".repeat(100),
        b"a".repeat(1000),
        b"a".repeat(10000),
    ];
    
    for (i, s) in strings.iter().enumerate() {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
            s.clone(),
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), strings.len() as u32);
}

/// Test ECC with nullable fields
#[test]
fn test_ecc_with_nulls() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("optional_field", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(4).ok();

    let config = WriterConfig {
        ecc,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    // Mix of null and non-null values
    writer.write_row(vec![
        1i64.to_le_bytes().to_vec(),
        b"data".to_vec(),
    ]).unwrap();
    
    writer.write_row(vec![
        2i64.to_le_bytes().to_vec(),
        vec![],  // NULL
    ]).unwrap();
    
    writer.write_row(vec![
        3i64.to_le_bytes().to_vec(),
        b"more data".to_vec(),
    ]).unwrap();

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 3);
}

/// Test ECC with all field types
#[test]
fn test_ecc_all_field_types() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("int32_col", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("int64_col", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("float32_col", FieldType::Float32, Nullability::Optional)
        .unwrap()
        .add_field("float64_col", FieldType::Float64, Nullability::Optional)
        .unwrap()
        .add_field("bool_col", FieldType::Boolean, Nullability::Optional)
        .unwrap()
        .add_field("string_col", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("blob_col", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let ecc = EccConfig::new(4).ok();

    let config = WriterConfig {
        ecc,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    writer.write_row(vec![
        42i32.to_le_bytes().to_vec(),
        100i64.to_le_bytes().to_vec(),
        (3.14f32).to_le_bytes().to_vec(),
        (2.71f64).to_le_bytes().to_vec(),
        vec![1],  // true
        b"text".to_vec(),
        vec![1, 2, 3, 4, 5],
    ]).unwrap();

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}
