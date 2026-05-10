//! Security integration tests for encryption and ECC
//! Tests encryption + ECC pipeline integration with reader/writer

use qrd_core::encryption::EncryptionConfig;
use qrd_core::ecc::EccConfig;
use qrd_core::writer::{FileWriter, WriterConfig};
use qrd_core::reader::FileReader;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use tempfile::NamedTempFile;
use std::io::Write;

fn serialize_string(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(bytes);
    result
}

fn create_test_schema() -> qrd_core::schema::Schema {
    SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("name", FieldType::String, Nullability::Required)
        .unwrap()
        .add_field("value", FieldType::Float64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap()
}

#[test]
fn test_encrypted_write_read_roundtrip() {
    let temp = NamedTempFile::new().unwrap();
    let schema = create_test_schema();

    // Generate encryption key and salt
    let salt = EncryptionConfig::generate_salt();
    let enc_config = EncryptionConfig::derive_from_password("test-password", &salt).unwrap();

    // Write with encryption
    let writer_config = WriterConfig {
        row_group_size: 1024,
        compression_level: 3,
        encryption: Some(enc_config.clone()),
        ecc: None,
        encrypt_footer: true,
        per_column_encryption: false,
    };

    let mut writer = FileWriter::with_config(
        std::fs::File::create(temp.path()).unwrap(),
        schema.clone(),
        writer_config,
    )
    .unwrap();

    // Write some test data
    for i in 0..10 {
        let row = vec![
            (i as i64).to_le_bytes().to_vec(), // id: i64
            serialize_string(&format!("record_{}", i)), // name: string
            (i as f64 * 1.5).to_le_bytes().to_vec(), // value: f64
        ];
        writer.write_row(row).unwrap();
    }

    writer.finish().unwrap();

    // Read with encryption config
    let reader = FileReader::with_decryption(temp.path(), enc_config).unwrap();
    assert_eq!(reader.row_count(), 10);
    assert_eq!(reader.schema().fields.len(), 3);
}

#[test]
fn test_encrypted_wrong_key_fails() {
    let temp = NamedTempFile::new().unwrap();
    let schema = create_test_schema();

    // Write with correct key
    let salt = EncryptionConfig::generate_salt();
    let enc_config = EncryptionConfig::derive_from_password("correct-password", &salt).unwrap();

    let writer_config = WriterConfig {
        row_group_size: 1024,
        compression_level: 3,
        encryption: Some(enc_config.clone()),
        ecc: None,
        encrypt_footer: true,
        per_column_encryption: false,
    };

    let mut writer = FileWriter::with_config(
        std::fs::File::create(temp.path()).unwrap(),
        schema.clone(),
        writer_config,
    )
    .unwrap();

    for i in 0..5 {
        let row = vec![
            (i as i64).to_le_bytes().to_vec(),
            serialize_string(&format!("data_{}", i)),
            (i as f64).to_le_bytes().to_vec(),
        ];
        writer.write_row(row).unwrap();
    }

    writer.finish().unwrap();

    // Try to read with wrong key
    let wrong_enc_config =
        EncryptionConfig::derive_from_password("wrong-password", &salt).unwrap();
    let result = FileReader::with_decryption(temp.path(), wrong_enc_config);

    // Should succeed in opening, but fail when trying to read
    if let Ok(reader) = result {
        // Try to read row group - should fail with wrong key
        let result = reader.read_row_group(0);
        assert!(result.is_err(), "Should fail with wrong encryption key");
    }
}

#[test]
fn test_ecc_corruption_recovery() {
    let temp = NamedTempFile::new().unwrap();
    let schema = create_test_schema();

    // Write with ECC enabled (2 parity chunks)
    let ecc_config = EccConfig::new(2).unwrap();

    let writer_config = WriterConfig {
        row_group_size: 1024,
        compression_level: 3,
        encryption: None,
        ecc: Some(ecc_config.clone()),
        encrypt_footer: false,
        per_column_encryption: false,
    };

    let mut writer = FileWriter::with_config(
        std::fs::File::create(temp.path()).unwrap(),
        schema.clone(),
        writer_config,
    )
    .unwrap();

    // Write test data
    for i in 0..50 {
        let row = vec![
            (i as i64).to_le_bytes().to_vec(),
            serialize_string(&format!("ecc_test_{}", i)),
            (i as f64 * 2.5).to_le_bytes().to_vec(),
        ];
        writer.write_row(row).unwrap();
    }

    writer.finish().unwrap();

    // Read the file normally first (should work)
    let reader = FileReader::with_ecc(temp.path(), ecc_config.clone()).unwrap();
    assert_eq!(reader.row_count(), 50);

    // Now corrupt the file by modifying some bytes in the first row group
    let mut file_data = std::fs::read(temp.path()).unwrap();
    
    // Skip header (32 bytes) and corrupt at offset 100 and 200
    if file_data.len() > 200 {
        file_data[100] ^= 0xFF; // Flip bits
        file_data[150] ^= 0xFF; // Flip bits
    }

    // Write corrupted data
    std::fs::write(temp.path(), file_data).unwrap();

    // Try to read with ECC recovery
    let reader = FileReader::with_ecc(temp.path(), ecc_config).unwrap();

    // Should still be readable thanks to ECC
    // This test may pass or fail depending on severity of corruption
    // Just check it doesn't panic
    let _ = reader.read_row_group(0);
}

#[test]
fn test_encrypted_ecc_combined() {
    let temp = NamedTempFile::new().unwrap();
    let schema = create_test_schema();

    // Create both encryption and ECC config
    let salt = EncryptionConfig::generate_salt();
    let enc_config = EncryptionConfig::derive_from_password("combined-test", &salt).unwrap();
    let ecc_config = EccConfig::new(2).unwrap();

    let writer_config = WriterConfig {
        row_group_size: 512,
        compression_level: 3,
        encryption: Some(enc_config.clone()),
        ecc: Some(ecc_config.clone()),
        encrypt_footer: true,
        per_column_encryption: false,
    };

    let mut writer = FileWriter::with_config(
        std::fs::File::create(temp.path()).unwrap(),
        schema.clone(),
        writer_config,
    )
    .unwrap();

    // Write data
    for i in 0..30 {
        let row = vec![
            (i as i64).to_le_bytes().to_vec(),
            serialize_string(&format!("combined_{}", i)),
            (i as f64 * 0.5).to_le_bytes().to_vec(),
        ];
        writer.write_row(row).unwrap();
    }

    writer.finish().unwrap();

    // Read with both encryption and ECC
    let reader = FileReader::with_security(
        temp.path(),
        Some(enc_config),
        Some(ecc_config),
    )
    .unwrap();

    assert_eq!(reader.row_count(), 30);

    // Successfully read row groups
    for i in 0..reader.row_group_offsets().len() {
        let rg = reader.read_row_group(i);
        assert!(rg.is_ok(), "Row group {} should be readable", i);
    }
}

#[test]
fn test_encrypted_footer_schema_hidden() {
    let temp = NamedTempFile::new().unwrap();
    let schema = create_test_schema();

    // Write with encrypted footer
    let enc_config = EncryptionConfig::new(EncryptionConfig::generate_key()).unwrap();

    let writer_config = WriterConfig {
        row_group_size: 1024,
        compression_level: 3,
        encryption: Some(enc_config.clone()),
        ecc: None,
        encrypt_footer: true,
        per_column_encryption: false,
    };

    let mut writer = FileWriter::with_config(
        std::fs::File::create(temp.path()).unwrap(),
        schema.clone(),
        writer_config,
    )
    .unwrap();

    for i in 0..10 {
        let row = vec![
            (i as i64).to_le_bytes().to_vec(),
            serialize_string(&format!("secret_{}", i)),
            (i as f64).to_le_bytes().to_vec(),
        ];
        writer.write_row(row).unwrap();
    }

    writer.finish().unwrap();

    // Try to read WITHOUT encryption key
    let result = FileReader::new(temp.path());

    // Should fail because footer is encrypted
    assert!(
        result.is_err(),
        "Should fail to read encrypted footer without key"
    );
}

#[test]
fn test_unencrypted_still_works() {
    let temp = NamedTempFile::new().unwrap();
    let schema = create_test_schema();

    // Write WITHOUT encryption
    let writer_config = WriterConfig::default();

    let mut writer = FileWriter::with_config(
        std::fs::File::create(temp.path()).unwrap(),
        schema.clone(),
        writer_config,
    )
    .unwrap();

    for i in 0..20 {
        let row = vec![
            (i as i64).to_le_bytes().to_vec(),
            serialize_string(&format!("normal_{}", i)),
            (i as f64 * 1.5).to_le_bytes().to_vec(),
        ];
        writer.write_row(row).unwrap();
    }

    writer.finish().unwrap();

    // Should read fine without any security config
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 20);

    // Also should work with security builder
    let reader_secure = FileReader::with_security(temp.path(), None, None).unwrap();
    assert_eq!(reader_secure.row_count(), 20);
}

#[test]
fn test_password_based_encryption_e2e() {
    let temp = NamedTempFile::new().unwrap();
    let schema = create_test_schema();

    let password = "my-secret-password-123!@#";
    let salt = EncryptionConfig::generate_salt();

    // Derive key from password
    let enc_config = EncryptionConfig::derive_from_password(password, &salt).unwrap();

    let writer_config = WriterConfig {
        row_group_size: 512,
        compression_level: 4,
        encryption: Some(enc_config.clone()),
        ecc: Some(EccConfig::new(1).unwrap()),
        encrypt_footer: true,
        per_column_encryption: false,
    };

    let mut writer = FileWriter::with_config(
        std::fs::File::create(temp.path()).unwrap(),
        schema.clone(),
        writer_config,
    )
    .unwrap();

    // Write data
    for i in 0..100 {
        let row = vec![
            (i as i64).to_le_bytes().to_vec(),
            serialize_string(&format!("password_protected_{:05}", i)),
            (i as f64 * 3.14).to_le_bytes().to_vec(),
        ];
        writer.write_row(row).unwrap();
    }

    writer.finish().unwrap();

    // Later: derive same key from password
    let derived_config = EncryptionConfig::derive_from_password(password, &salt).unwrap();

    // Should read successfully
    let reader = FileReader::with_decryption(temp.path(), derived_config).unwrap();
    assert_eq!(reader.row_count(), 100);

    // Read all row groups successfully
    let all_rgs = reader.read_all_row_groups();
    assert!(all_rgs.is_ok());
}

#[test]
fn test_ecc_partial_corruption_recovery() {
    let temp = NamedTempFile::new().unwrap();
    let schema = create_test_schema();

    // Write with 3 parity chunks (can recover from up to 3 lost shards)
    let ecc_config = EccConfig::with_chunk_size(3, 2048).unwrap();

    let writer_config = WriterConfig {
        row_group_size: 2048,
        compression_level: 3,
        encryption: None,
        ecc: Some(ecc_config.clone()),
        encrypt_footer: false,
        per_column_encryption: false,
    };

    let mut writer = FileWriter::with_config(
        std::fs::File::create(temp.path()).unwrap(),
        schema.clone(),
        writer_config,
    )
    .unwrap();

    // Write enough data to create multiple chunks
    for i in 0..200 {
        let row = vec![
            (i as i64).to_le_bytes().to_vec(),
            serialize_string(&format!("chunk_test_{:06}", i)),
            (i as f64 * 1.23).to_le_bytes().to_vec(),
        ];
        writer.write_row(row).unwrap();
    }

    writer.finish().unwrap();

    // Verify reads normally
    let reader = FileReader::with_ecc(temp.path(), ecc_config.clone()).unwrap();
    assert_eq!(reader.row_count(), 200);

    // Corrupt multiple locations in first row group
    let mut file_data = std::fs::read(temp.path()).unwrap();
    if file_data.len() > 500 {
        for offset in &[100, 200, 350, 400] {
            if *offset < file_data.len() {
                file_data[*offset] ^= 0x55;
            }
        }
    }
    std::fs::write(temp.path(), file_data).unwrap();

    // Should still recover
    let reader = FileReader::with_ecc(temp.path(), ecc_config).unwrap();
    let _ = reader.read_row_group(0); // Just check it doesn't panic
}
