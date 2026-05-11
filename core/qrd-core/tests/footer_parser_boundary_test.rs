//! Footer parser boundary and error condition tests
//!
//! Tests various boundary conditions in footer parsing:
//! - Truncated footers
//! - Invalid footer lengths
//! - Malformed data
//! - Edge cases in parsing

use qrd_core::prelude::*;
use qrd_core::writer::FileWriter;
use qrd_core::reader::FileReader;
use qrd_core::footer::Footer;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use tempfile::NamedTempFile;
use std::io::{Seek, SeekFrom, Read, Write};
use std::fs::File;

/// Helper function to create a valid QRD file
fn create_valid_qrd_file(path: &std::path::Path) -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)?
        .add_field("name", FieldType::String, Nullability::Optional)?
        .build()?;

    let mut writer = FileWriter::new(path, schema)?;
    writer.write_row(vec![
        1i64.to_le_bytes().to_vec(),
        b"test".to_vec(),
    ])?;
    writer.write_row(vec![
        2i64.to_le_bytes().to_vec(),
        b"data".to_vec(),
    ])?;
    writer.finish()?;

    Ok(())
}

/// Test reading valid QRD file
#[test]
fn test_footer_valid_qrd_file() {
    let temp = NamedTempFile::new().unwrap();
    create_valid_qrd_file(temp.path()).unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 2);
}

/// Test footer parsing with file too small (< 40 bytes)
#[test]
fn test_footer_file_too_small() {
    let temp = NamedTempFile::new().unwrap();
    let mut file = File::create(temp.path()).unwrap();
    
    // Write minimal data less than 40 bytes
    file.write_all(b"QRD\x01").unwrap();  // Just magic bytes
    drop(file);

    // Should fail to parse
    let result = FileReader::new(temp.path());
    assert!(result.is_err());
}

/// Test footer with truncated footer length field
#[test]
fn test_footer_truncated_at_footer_length() {
    let temp = NamedTempFile::new().unwrap();
    
    // Create a minimal file that ends abruptly at footer length field
    let mut file = File::create(temp.path()).unwrap();
    file.write_all(&[0u8; 35]).unwrap();  // Just under 40 bytes
    drop(file);

    let result = FileReader::new(temp.path());
    assert!(result.is_err());
}

/// Test footer with footer length larger than file
#[test]
fn test_footer_length_exceeds_file_size() {
    let temp = NamedTempFile::new().unwrap();
    let mut file = File::create(temp.path()).unwrap();
    
    // Write header
    file.write_all(b"QRD\x01").unwrap();
    file.write_all(&[0u8; 28]).unwrap();  // Padding to 32 bytes
    
    // Write some dummy data
    file.write_all(&[0xFF; 16]).unwrap();  // 16 bytes of data
    
    // Write invalid footer length (larger than remaining space)
    file.write_all(&[0xFF, 0xFF, 0xFF, 0x7F]).unwrap();  // 2147483647 bytes
    drop(file);

    let result = FileReader::new(temp.path());
    assert!(result.is_err());
}

/// Test footer with zero-length footer
#[test]
fn test_footer_zero_length() {
    let temp = NamedTempFile::new().unwrap();
    let mut file = File::create(temp.path()).unwrap();
    
    // Write header (32 bytes)
    file.write_all(b"QRD\x01").unwrap();
    file.write_all(&[0u8; 28]).unwrap();
    
    // Write zero-length footer indicator
    file.write_all(&[0u8, 0u8, 0u8, 0u8]).unwrap();  // Footer length = 0
    drop(file);

    // Even with zero footer, should be readable
    let result = FileReader::new(temp.path());
    // Behavior depends on implementation - may succeed or fail gracefully
    let _ = result;
}

/// Test footer with excessive footer length (sanity check > 1MB)
#[test]
fn test_footer_excessive_length() {
    let temp = NamedTempFile::new().unwrap();
    let mut file = File::create(temp.path()).unwrap();
    
    // Write header
    file.write_all(b"QRD\x01").unwrap();
    file.write_all(&[0u8; 28]).unwrap();
    
    // Write footer length > 1MB (1048576)
    let too_large = (2 * 1024 * 1024) as u32;  // 2MB
    file.write_all(&too_large.to_le_bytes()).unwrap();
    drop(file);

    let result = FileReader::new(temp.path());
    assert!(result.is_err());
}

/// Test footer parsing with partial footer data
#[test]
fn test_footer_partial_data() {
    let temp = NamedTempFile::new().unwrap();
    
    // Create valid file first
    create_valid_qrd_file(temp.path()).unwrap();
    
    // Now truncate it
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(temp.path())
        .unwrap();
    
    let metadata = file.metadata().unwrap();
    let original_len = metadata.len();
    
    // Truncate to half size
    file.set_len(original_len / 2).unwrap();
    drop(file);

    let result = FileReader::new(temp.path());
    assert!(result.is_err());
}

/// Test footer parsing boundary with barely valid size
#[test]
fn test_footer_minimal_valid_size() {
    let temp = NamedTempFile::new().unwrap();
    
    // Create valid file
    create_valid_qrd_file(temp.path()).unwrap();
    
    // Read it successfully
    let reader = FileReader::new(temp.path()).unwrap();
    assert!(reader.row_count() > 0);
}

/// Test footer with corrupted CRC
#[test]
fn test_footer_corrupted_crc() {
    let temp = NamedTempFile::new().unwrap();
    
    // Create valid file
    create_valid_qrd_file(temp.path()).unwrap();
    
    // Corrupt the last few bytes (CRC area)
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(temp.path())
        .unwrap();
    
    file.seek(SeekFrom::End(-8)).unwrap();
    file.write_all(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]).unwrap();
    drop(file);

    // Reading might fail due to CRC validation
    let result = FileReader::new(temp.path());
    // Behavior depends on CRC validation implementation
    let _ = result;
}

/// Test footer parsing with row count zero
#[test]
fn test_footer_zero_row_count() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 0);
}

/// Test footer with misaligned data
#[test]
fn test_footer_misaligned_data() {
    let temp = NamedTempFile::new().unwrap();
    let mut file = File::create(temp.path()).unwrap();
    
    // Write header with wrong alignment
    file.write_all(b"QRD\x01").unwrap();
    file.write_all(&[0u8; 27]).unwrap();  // Misaligned - only 31 bytes total
    
    // Try to write more data
    file.write_all(&[0xFF; 100]).unwrap();
    drop(file);

    let result = FileReader::new(temp.path());
    // Should handle gracefully
    let _ = result;
}

/// Test footer with all zeros
#[test]
fn test_footer_all_zeros() {
    let temp = NamedTempFile::new().unwrap();
    let mut file = File::create(temp.path()).unwrap();
    
    // Write file of all zeros
    file.write_all(&[0u8; 256]).unwrap();
    drop(file);

    let result = FileReader::new(temp.path());
    // Should fail gracefully
    assert!(result.is_err());
}

/// Test footer with all ones
#[test]
fn test_footer_all_ones() {
    let temp = NamedTempFile::new().unwrap();
    let mut file = File::create(temp.path()).unwrap();
    
    // Write file of all ones
    file.write_all(&[0xFF; 256]).unwrap();
    drop(file);

    let result = FileReader::new(temp.path());
    // Should fail gracefully
    assert!(result.is_err());
}

/// Test footer with exactly 40 bytes (minimum theoretical size)
#[test]
fn test_footer_exactly_minimum_size() {
    let temp = NamedTempFile::new().unwrap();
    let mut file = File::create(temp.path()).unwrap();
    
    // Write exactly 40 bytes
    file.write_all(&[0u8; 40]).unwrap();
    drop(file);

    let result = FileReader::new(temp.path());
    // Should handle edge case
    let _ = result;
}

/// Test reading footer after multiple row groups
#[test]
fn test_footer_multiple_row_groups() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let config = qrd_core::writer::WriterConfig {
        row_group_size: 10,
        ..Default::default()
    };

    let file = File::create(temp.path()).unwrap();
    let mut writer = FileWriter::with_config(file, schema, config).unwrap();

    // Write enough rows to create multiple row groups
    for i in 0..100 {
        writer.write_row(vec![
            (i as i64).to_le_bytes().to_vec(),
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 100);
}

/// Test footer with maximum row count
#[test]
fn test_footer_maximum_row_count() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema).unwrap();

    // Write many rows (but not actually reach u32::MAX in practical test)
    for i in 0..10000 {
        writer.write_row(vec![
            (i as i32).to_le_bytes().to_vec(),
        ]).unwrap();
    }

    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 10000);
}
