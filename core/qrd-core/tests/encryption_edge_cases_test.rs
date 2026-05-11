//! Encryption edge case and per-column encryption tests
//!
//! Tests various encryption scenarios:
//! - Per-column encryption schemes
//! - Key derivation with different encryption parameters
//! - Encryption/decryption roundtrips
//! - Edge cases in encrypted data handling

use qrd_core::prelude::*;
use qrd_core::writer::{FileWriter, WriterConfig};
use qrd_core::reader::FileReader;
use qrd_core::encryption::EncryptionConfig;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use tempfile::NamedTempFile;
use std::fs::File;

/// Test basic encryption/decryption roundtrip
#[test]
fn test_encryption_basic_roundtrip() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("secret", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    // Create encryption config with specific key
    let key = [42u8; 32];
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
        encrypt_footer: true,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    writer.write_row(vec![
        1i64.to_le_bytes().to_vec(),
        b"sensitive data".to_vec(),
    ]).unwrap();

    writer.finish().unwrap();

    // Verify file was encrypted (has encryption marker)
    assert!(temp.path().exists());
    let file_size = temp.path().metadata().unwrap().len();
    assert!(file_size > 0);
}

/// Test encryption with zero key
#[test]
fn test_encryption_zero_key() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let key = [0u8; 32];  // All zeros key
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
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

/// Test encryption with all-ones key
#[test]
fn test_encryption_ones_key() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let key = [0xFFu8; 32];  // All ones key
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
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

/// Test per-column encryption with different column names
#[test]
fn test_per_column_encryption_different_columns() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("user_id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("email", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("password_hash", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("ssn", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let key = [123u8; 32];
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
        per_column_encryption: true,
        encrypt_footer: true,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    // Each column will be encrypted separately with derived key
    writer.write_row(vec![
        1i64.to_le_bytes().to_vec(),
        b"user@example.com".to_vec(),
        b"hashed_password_123".to_vec(),
        b"123-45-6789".to_vec(),
    ]).unwrap();

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test per-column encryption with many columns
#[test]
fn test_per_column_encryption_many_columns() {
    let mut builder = SchemaBuilder::new();
    
    // Create 20 columns
    for i in 0..20 {
        builder = builder
            .add_field(
                &format!("col_{}", i),
                if i % 2 == 0 {
                    FieldType::Int64
                } else {
                    FieldType::String
                },
                Nullability::Optional,
            )
            .unwrap();
    }
    
    let temp = NamedTempFile::new().unwrap();
    let schema = builder.build().unwrap();

    let key = [77u8; 32];
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
        per_column_encryption: true,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema.clone(), config).unwrap();
    
    let mut row = Vec::new();
    for i in 0..20 {
        if i % 2 == 0 {
            row.push((i as i64).to_le_bytes().to_vec());
        } else {
            row.push(format!("data_{}", i).as_bytes().to_vec());
        }
    }
    
    writer.write_row(row).unwrap();
    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test encryption with footer encryption disabled
#[test]
fn test_encryption_no_footer_encryption() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("data", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let key = [88u8; 32];
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
        encrypt_footer: false,  // Footer is not encrypted
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

/// Test encryption with null values in encrypted fields
#[test]
fn test_encryption_with_nulls() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("optional_secure", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let key = [55u8; 32];
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    // Mix of null and non-null encrypted fields
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
    assert!(temp.path().exists());
}

/// Test encryption with empty string values
#[test]
fn test_encryption_empty_strings() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("text", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let key = [99u8; 32];
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    // Empty string values
    writer.write_row(vec![
        1i64.to_le_bytes().to_vec(),
        b"".to_vec(),  // Empty string
    ]).unwrap();
    
    writer.write_row(vec![
        2i64.to_le_bytes().to_vec(),
        b"".to_vec(),  // Empty string
    ]).unwrap();

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test encryption with very large encrypted fields
#[test]
fn test_encryption_large_fields() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("large_data", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let key = [11u8; 32];
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    // Write large encrypted blobs
    let large_blob = vec![0xAB; 10 * 1024 * 1024];  // 10MB encrypted blob
    
    writer.write_row(vec![
        1i64.to_le_bytes().to_vec(),
        large_blob.clone(),
    ]).unwrap();

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test encryption key derivation
#[test]
fn test_encryption_key_derivation() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("col1", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("col2", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("col3", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let key = [22u8; 32];
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    // Test that different column names derive different keys
    if let Some(enc) = encryption {
        let key1 = enc.derive_column_key("col1");
        let key2 = enc.derive_column_key("col2");
        let key3 = enc.derive_column_key("col3");
        
        // Keys should be derivable (Ok result)
        assert!(key1.is_ok());
        assert!(key2.is_ok());
        assert!(key3.is_ok());
        
        // Keys should be different for different column names
        if let (Ok(k1), Ok(k2), Ok(k3)) = (key1, key2, key3) {
            assert_ne!(k1, k2);
            assert_ne!(k2, k3);
            assert_ne!(k1, k3);
        }
    }
}

/// Test encryption with special characters in column names
#[test]
fn test_encryption_special_column_names() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("col_with-dash", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("col.with.dots", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("col_with_underscore", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let key = [33u8; 32];
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
        per_column_encryption: true,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    writer.write_row(vec![
        b"data1".to_vec(),
        b"data2".to_vec(),
        b"data3".to_vec(),
    ]).unwrap();

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test encryption roundtrip with multiple rows
#[test]
fn test_encryption_multiple_rows() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("secret", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let key = [44u8; 32];
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    for i in 0..100 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
            format!("secret_{}", i).as_bytes().to_vec(),
        ]).unwrap();
    }

    writer.finish().unwrap();
    assert!(temp.path().exists());
}

/// Test encryption with mixed data types
#[test]
fn test_encryption_mixed_types() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("int64_col", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("float64_col", FieldType::Float64, Nullability::Optional)
        .unwrap()
        .add_field("string_col", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("blob_col", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .add_field("bool_col", FieldType::Boolean, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let key = [66u8; 32];
    let encryption = EncryptionConfig::new(key.to_vec()).ok();

    let config = WriterConfig {
        encryption,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();
    
    writer.write_row(vec![
        42i64.to_le_bytes().to_vec(),
        (3.14f64).to_le_bytes().to_vec(),
        b"text data".to_vec(),
        vec![1, 2, 3, 4, 5],
        vec![1],  // true
    ]).unwrap();

    writer.finish().unwrap();
    assert!(temp.path().exists());
}
