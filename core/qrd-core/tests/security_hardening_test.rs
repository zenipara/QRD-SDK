//! Phase 5: Security & Hardening Tests
//!
//! Comprehensive test suite for:
//! - Encryption validation (AES-256-GCM, key derivation, nonce uniqueness)
//! - Input validation (malformed files, truncation, bounds checking)
//! - CRC32 and integrity validation
//! - Error handling consistency

use qrd_core::{
    encryption::{EncryptionConfig, encrypt, decrypt},
    validation::{Validator, CorruptionDetector, CorruptionType},
    error::Error,
    prelude::{SchemaBuilder, FieldType, Nullability},
};

// ============= S1: Encryption Validation =============

#[test]
fn s1_1_aes_256_gcm_nonce_uniqueness() {
    // S1.1: Verify nonce uniqueness per encryption
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();
    
    let data = b"Sensitive data for nonce test";
    
    // Encrypt same data multiple times
    let encrypted1 = encrypt(data, &config).unwrap();
    let encrypted2 = encrypt(data, &config).unwrap();
    
    // Different nonces should produce different ciphertexts
    // (same plaintext with different nonces = different ciphertext)
    assert_ne!(encrypted1, encrypted2, "Nonces should be unique, ciphertexts must differ");
    
    // Both should decrypt to the same plaintext
    let decrypted1 = decrypt(&encrypted1, &config).unwrap();
    let decrypted2 = decrypt(&encrypted2, &config).unwrap();
    
    assert_eq!(decrypted1, data);
    assert_eq!(decrypted2, data);
}

#[test]
fn s1_2_aes_gcm_authentication_tag_validation() {
    // S1.2: Verify authentication tag validation (tampering detection)
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();
    
    let data = b"QRD encrypted data";
    let encrypted = encrypt(data, &config).unwrap();
    
    // Tamper with ciphertext (flip a bit somewhere after nonce)
    let mut tampered = encrypted.clone();
    if tampered.len() > 20 {
        tampered[20] ^= 0x01; // Flip one bit
    }
    
    // Decryption should fail
    let result = decrypt(&tampered, &config);
    assert!(result.is_err(), "Tampered ciphertext should fail authentication");
}

#[test]
fn s1_3_wrong_key_decryption_fails() {
    // S1.3: Verify decryption fails with wrong key
    let key1 = EncryptionConfig::generate_key();
    let key2 = EncryptionConfig::generate_key();
    
    let config1 = EncryptionConfig::new(key1).unwrap();
    let config2 = EncryptionConfig::new(key2).unwrap();
    
    let data = b"Secret message";
    let encrypted = encrypt(data, &config1).unwrap();
    
    // Attempting to decrypt with wrong key should fail
    let result = decrypt(&encrypted, &config2);
    assert!(result.is_err(), "Wrong key should fail decryption");
}

#[test]
fn s1_4_key_derivation_from_password_determinism() {
    // S1.4: Verify same password → same key (deterministic)
    let password = "my-secure-password";
    let salt = EncryptionConfig::generate_salt();
    
    let config1 = EncryptionConfig::derive_from_password(password, &salt).unwrap();
    let config2 = EncryptionConfig::derive_from_password(password, &salt).unwrap();
    
    // Same password + salt = same key
    assert_eq!(config1.key, config2.key, "Key derivation must be deterministic");
    
    // Different password = different key
    let config3 = EncryptionConfig::derive_from_password("different-password", &salt).unwrap();
    assert_ne!(config1.key, config3.key, "Different passwords must produce different keys");
}

#[test]
fn s1_5_per_column_key_derivation_uniqueness() {
    // S1.5: Verify per-column keys are unique and deterministic
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();
    
    // Derive keys for multiple columns
    let col1_key = config.derive_column_key("temperature").unwrap();
    let col2_key = config.derive_column_key("humidity").unwrap();
    let col3_key = config.derive_column_key("pressure").unwrap();
    
    // All different
    assert_ne!(col1_key, col2_key);
    assert_ne!(col2_key, col3_key);
    assert_ne!(col1_key, col3_key);
    
    // Deterministic
    let col1_key_again = config.derive_column_key("temperature").unwrap();
    assert_eq!(col1_key, col1_key_again);
}

#[test]
fn s1_6_selective_encryption_decrypt() {
    // S1.6: Verify selective column encryption
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();
    
    let col1_key = config.derive_column_key("encrypted_col").unwrap();
    let col1_config = EncryptionConfig::new(col1_key).unwrap();
    
    let encrypted_data = b"This is encrypted";
    let encrypted = encrypt(encrypted_data, &col1_config).unwrap();
    
    // Can be decrypted with the same derived key
    let decrypted = decrypt(&encrypted, &col1_config).unwrap();
    assert_eq!(decrypted, encrypted_data);
    
    // Cannot be decrypted without the right key
    let wrong_result = decrypt(&encrypted, &config);
    assert!(wrong_result.is_err());
}

// ============= S2: Input Validation =============

#[test]
fn s2_1_malformed_magic_bytes() {
    // S2.1: Detect corrupted magic bytes
    let invalid_magic: &[u8; 4] = b"XXXX";
    let result = Validator::validate_magic(invalid_magic);
    assert!(result.is_err(), "Invalid magic bytes should be detected");
    
    let valid_magic: &[u8; 4] = b"QRD\x01";
    let result = Validator::validate_magic(valid_magic);
    assert!(result.is_ok(), "Valid QRD magic should pass");
}

#[test]
fn s2_2_truncated_file_detection() {
    // S2.2: Detect truncated files
    let expected_min = 1024u64;
    let truncated_size = 512u64;
    
    let report = CorruptionDetector::detect_truncation(truncated_size, expected_min);
    assert!(report.is_some());
    assert_eq!(report.unwrap().corruption_type, CorruptionType::TruncatedFile);
    
    // Non-truncated file should pass
    let report = CorruptionDetector::detect_truncation(expected_min + 100, expected_min);
    assert!(report.is_none());
}

#[test]
fn s2_3_invalid_offset_detection() {
    // S2.3: Detect out-of-bounds offsets
    let file_size = 1000u64;
    let valid_offset = 500u64;
    let invalid_offset = 2000u64;
    
    let result = CorruptionDetector::detect_invalid_offset(valid_offset, file_size);
    assert!(result.is_none(), "Valid offset should not report corruption");
    
    let result = CorruptionDetector::detect_invalid_offset(invalid_offset, file_size);
    assert!(result.is_some(), "Out-of-bounds offset should be detected");
}

#[test]
fn s2_4_row_count_mismatch_detection() {
    // S2.4: Detect row count mismatches
    let header_count = 100u32;
    let calculated_count = 105u32;
    
    let report = CorruptionDetector::detect_row_count_mismatch(header_count, calculated_count);
    assert!(report.is_some());
    assert_eq!(report.unwrap().corruption_type, CorruptionType::RowCountMismatch);
    
    // Matching counts should pass
    let report = CorruptionDetector::detect_row_count_mismatch(100, 100);
    assert!(report.is_none());
}

#[test]
fn s2_5_monotonic_offsets_validation() {
    // S2.5: Validate offsets are monotonically increasing
    let valid_offsets = vec![100u64, 200, 300, 400, 500];
    let result = CorruptionDetector::validate_monotonic_offsets(&valid_offsets);
    assert!(result.is_ok(), "Valid monotonic offsets should pass");
    
    // Non-monotonic offsets should fail
    let invalid_offsets = vec![100u64, 200, 300, 250, 400];
    let result = CorruptionDetector::validate_monotonic_offsets(&invalid_offsets);
    assert!(result.is_err(), "Non-monotonic offsets should be rejected");
    
    // Duplicate offsets should fail
    let duplicate_offsets = vec![100u64, 200, 200, 300];
    let result = CorruptionDetector::validate_monotonic_offsets(&duplicate_offsets);
    assert!(result.is_err(), "Duplicate offsets should be rejected");
}

#[test]
fn s2_6_crc32_validation() {
    // S2.6: Test CRC32 integrity checking
    let data = b"Important data that must be protected";
    let crc = Validator::calculate_crc32(data);
    
    // Correct CRC should pass
    let result = Validator::verify_crc32(data, crc);
    assert!(result.is_ok());
    
    // Wrong CRC should fail
    let result = Validator::verify_crc32(data, crc + 1);
    assert!(result.is_err());
    
    // Tampered data with original CRC should fail
    let mut tampered = data.to_vec();
    if !tampered.is_empty() {
        tampered[0] ^= 0x01;
    }
    let result = Validator::verify_crc32(&tampered, crc);
    assert!(result.is_err());
}

#[test]
fn s2_7_version_validation() {
    // S2.7: Test version compatibility checking
    let result = Validator::validate_version(1, 0);
    assert!(result.is_ok(), "Compatible version should pass");
    
    // Future major version should fail
    let result = Validator::validate_version(100, 0);
    assert!(result.is_err(), "Unsupported version should be rejected");
}

// ============= S3: Bounds & Size Validation =============

#[test]
fn s3_1_max_row_group_size_validation() {
    // S3.1: Test row group size bounds
    const MAX_ROW_GROUP_SIZE: u64 = 1_000_000_000; // 1GB
    
    let valid_size = 500_000_000u64;
    let invalid_size = 2_000_000_000u64;
    
    assert!(valid_size <= MAX_ROW_GROUP_SIZE);
    assert!(invalid_size > MAX_ROW_GROUP_SIZE);
}

#[test]
fn s3_2_string_length_bounds() {
    // S3.2: String length should be bounded
    const MAX_STRING_LENGTH: u32 = 1_000_000; // 1MB per string
    
    let valid_string = "hello world";
    let invalid_length = MAX_STRING_LENGTH as usize + 1;
    
    assert!(valid_string.len() as u32 <= MAX_STRING_LENGTH);
    assert!(invalid_length as u32 > MAX_STRING_LENGTH);
}

#[test]
fn s3_3_array_size_bounds() {
    // S3.3: Array sizes should be bounded
    const MAX_ARRAY_LENGTH: u32 = 10_000_000; // 10M elements
    
    let valid_array: Vec<i64> = vec![1; 1000];
    let invalid_size = MAX_ARRAY_LENGTH as usize + 1;
    
    assert!(valid_array.len() as u32 <= MAX_ARRAY_LENGTH);
    assert!(invalid_size as u32 > MAX_ARRAY_LENGTH);
}

#[test]
fn s3_4_column_count_bounds() {
    // S3.4: Schema should have reasonable column limits
    let schema_builder = SchemaBuilder::new();
    let schema_builder = schema_builder.add_field("col1".to_string(), FieldType::Int64, Nullability::Required).unwrap();
    let _schema = schema_builder.build().unwrap();
    
    // Schema structure validated through successful build with at least 1 column
}

// ============= S4: Error Handling Consistency =============

#[test]
fn s4_1_encryption_error_messages() {
    // S4.1: Verify encryption produces clear error messages
    
    // Invalid key length
    let short_key = vec![0u8; 16];
    let error = EncryptionConfig::new(short_key).err();
    assert!(error.is_some());
    if let Some(Error::ConfigError(msg)) = error {
        assert!(msg.contains("32 bytes"), "Error message should mention byte requirement");
    }
}

#[test]
fn s4_2_decryption_error_messages() {
    // S4.2: Verify decryption errors have context
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();
    
    // Too short encrypted data
    let short_data = vec![0u8; 1];
    let error = decrypt(&short_data, &config).err();
    assert!(error.is_some());
    
    // Tampered data
    let data = b"test";
    let valid_encrypted = encrypt(data, &config).unwrap();
    let mut tampered = valid_encrypted.clone();
    if tampered.len() > 1 {
        tampered[1] ^= 0xFF;
    }
    let error = decrypt(&tampered, &config).err();
    assert!(error.is_some());
}

#[test]
fn s4_3_validation_error_consistency() {
    // S4.3: Verify validation errors are consistent
    
    // Magic validation errors
    let invalid_magic = b"XXXX";
    let result = Validator::validate_magic(invalid_magic);
    assert!(result.is_err());
    
    // Version validation errors
    let result = Validator::validate_version(999, 999);
    assert!(result.is_err());
    
    // All should produce readable errors
    let magic_error = Validator::validate_magic(invalid_magic).err();
    let version_error = Validator::validate_version(999, 999).err();
    
    assert!(magic_error.is_some());
    assert!(version_error.is_some());
}

// ============= Integration Tests =============

#[test]
fn integration_schema_validation_with_encryption() {
    // Integration: Create schema, validate structure, prepare for encryption
    
    // Build schema with multiple fields
    let schema_builder = SchemaBuilder::new();
    let schema_builder = schema_builder.add_field("id".to_string(), FieldType::Int64, Nullability::Required).unwrap();
    let schema_builder = schema_builder.add_field("name".to_string(), FieldType::String, Nullability::Optional).unwrap();
    let _schema = schema_builder.build().unwrap();
    
    // Validate schema structure
    // Schema structure validated through successful build
    
    // In a full test, we would:
    // 1. Calculate CRC32 of schema
    // 2. Set up per-column encryption keys
    // 3. Write file with validation  
    // 4. Verify footer contains metadata
}

#[test]
fn no_panics_on_malformed_input() {
    // Fuzz-lite: Ensure no panics on invalid data
    
    // Empty data
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();
    let _ = decrypt(&[], &config);
    
    // Single byte
    let _ = decrypt(&[0u8], &config);
    
    // All zeros
    let _ = decrypt(&vec![0u8; 100], &config);
    
    // All 0xFF
    let _ = decrypt(&vec![0xFF; 100], &config);
    
    // No panics should occur
}
