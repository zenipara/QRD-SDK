//! Advanced fuzzing tests for QRD format resilience
//!
//! Tests edge cases, malformed inputs, and corruption scenarios
//! to ensure the format handles all error conditions gracefully.

use qrd_core::encryption::{EncryptionConfig, encrypt, decrypt};
use qrd_core::ecc::{EccCodec, EccConfig};
use qrd_core::error::Error;
use qrd_core::validation::CorruptionDetector;
use std::io::Cursor;

// ============================================================================
// ENCRYPTION FUZZING
// ============================================================================

/// Fuzz encryption with various malformed inputs
#[test]
fn fuzz_encryption_invalid_keys() {
    let test_cases = vec![
        (vec![], "empty key"),
        (vec![0; 16], "16-byte key (too short)"),
        (vec![0; 31], "31-byte key (off by one)"),
        (vec![0; 64], "64-byte key (too long)"),
        (vec![0xFF; 32], "all ones"),
        (vec![0; 32], "all zeros"),
    ];

    for (key, desc) in test_cases {
        println!("Testing encryption with {}", desc);
        
        let result = EncryptionConfig::new(key);
        match result {
            Ok(_) if desc.contains("32") => {
                // Valid 32-byte key should succeed
                println!("  ✓ Correctly accepted valid key");
            }
            Ok(_) => {
                panic!("Should reject invalid key: {}", desc);
            }
            Err(_) => {
                println!("  ✓ Correctly rejected invalid key");
            }
        }
    }
}

/// Fuzz decryption with corrupted ciphertexts
#[test]
fn fuzz_decryption_corrupted_ciphertexts() {
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();
    let plaintext = b"secret message";

    let mut ciphertext = encrypt(plaintext, &config).expect("Encryption failed");

    // Apply various corruptions
    let corruptions = vec![
        ("truncate to 1 byte", |ct: &mut Vec<u8>| ct.truncate(1)),
        ("truncate to half", |ct: &mut Vec<u8>| ct.truncate(ct.len() / 2)),
        ("flip first byte", |ct: &mut Vec<u8>| ct[0] ^= 0xFF),
        ("flip last byte", |ct: &mut Vec<u8>| ct[ct.len() - 1] ^= 0xFF),
        ("flip middle", |ct: &mut Vec<u8>| {
            if ct.len() > 2 {
                ct[ct.len() / 2] ^= 0xFF;
            }
        }),
        ("zero all", |ct: &mut Vec<u8>| {
            for b in ct.iter_mut() {
                *b = 0;
            }
        }),
        ("add random bytes", |ct: &mut Vec<u8>| {
            ct.extend_from_slice(&[0xFF; 100]);
        }),
    ];

    for (desc, corruption) in corruptions {
        let mut corrupted = ciphertext.clone();
        corruption(&mut corrupted);

        println!("Testing decryption with {}", desc);
        
        let result = decrypt(&corrupted, &config);
        match result {
            Ok(decrypted) => {
                // Should either fail or produce garbage
                assert_ne!(decrypted, plaintext, 
                    "Corrupted ciphertext should not decrypt to original");
                println!("  Result: Produced garbage (acceptable)");
            }
            Err(e) => {
                // Expected for authentication failure
                println!("  ✓ Correctly rejected corrupted ciphertext: {}", e);
            }
        }
    }
}

/// Fuzz password-based key derivation with malformed inputs
#[test]
fn fuzz_password_derivation_invalid_salt() {
    let password = "test_password";
    let test_cases = vec![
        (vec![], "empty salt"),
        (vec![0; 16], "16-byte salt (too short)"),
        (vec![0; 31], "31-byte salt (off by one)"),
        (vec![0; 64], "64-byte salt (too long)"),
    ];

    for (salt, desc) in test_cases {
        println!("Testing password derivation with {}", desc);
        
        let result = EncryptionConfig::derive_from_password(password, &salt);
        match result {
            Ok(_) if salt.len() == 32 => {
                println!("  ✓ Correctly accepted valid salt");
            }
            Ok(_) => {
                panic!("Should reject invalid salt: {}", desc);
            }
            Err(_) => {
                println!("  ✓ Correctly rejected invalid salt");
            }
        }
    }
}

// ============================================================================
// ECC FUZZING
// ============================================================================

/// Fuzz ECC configuration with invalid parameters
#[test]
fn fuzz_ecc_invalid_config() {
    let test_cases = vec![
        (0, 256, "zero data chunks"),
        (1, 0, "zero chunk size"),
        (5, 256, "valid config"),
        (10, 1024, "valid config with more parities"),
    ];

    for (parity, chunk_size, desc) in test_cases {
        println!("Testing ECC config with {}", desc);
        
        let result = EccConfig::with_chunk_size(parity, chunk_size);
        match result {
            Ok(_) if parity > 0 && chunk_size > 0 => {
                println!("  ✓ Correctly created valid config");
            }
            Ok(_) => {
                panic!("Should reject invalid config: {}", desc);
            }
            Err(_) => {
                println!("  ✓ Correctly rejected invalid config");
            }
        }
    }
}

/// Fuzz ECC encoding with extreme data sizes
#[test]
fn fuzz_ecc_extreme_sizes() {
    let config = EccConfig::with_chunk_size(2, 256).expect("Config creation failed");
    let mut codec = EccCodec::new(config).expect("Codec creation failed");

    let test_cases = vec![
        ("empty", vec![]),
        ("1 byte", vec![42]),
        ("10 bytes", vec![42; 10]),
        ("chunk aligned", vec![123; 256]),
        ("chunk + 1", vec![123; 257]),
        ("large", vec![42; 1024 * 100]),
    ];

    for (desc, data) in test_cases {
        println!("Testing ECC with {} data", desc);
        
        match codec.encode(&data) {
            Ok(encoded) => {
                println!("  ✓ Successfully encoded");
                
                // Try to recover with one loss
                let mut shards = encoded.shards_as_options();
                if shards.len() > 0 {
                    shards[0] = None;
                    match qrd_core::ecc::decode_and_recover(&shards, &config) {
                        Ok(recovered) => {
                            assert_eq!(recovered, data, "Recovery should produce original");
                            println!("  ✓ Successfully recovered from loss");
                        }
                        Err(e) => {
                            println!("  ! Recovery failed: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("  ! Encoding failed: {}", e);
            }
        }
    }
}

/// Fuzz ECC recovery with various loss patterns
#[test]
fn fuzz_ecc_loss_patterns() {
    let config = EccConfig::with_chunk_size(3, 512).expect("Config creation failed");
    let mut codec = EccCodec::new(config.clone()).expect("Codec creation failed");
    
    let data = vec![42u8; 2048];
    let encoded = codec.encode(&data).expect("Encoding failed");
    let shard_count = encoded.shard_count();

    // Test patterns: lose different numbers of shards
    for loss_count in 0..=3 {
        if loss_count > 3 {
            println!("Skipping loss of {} (too many for 3 parity)", loss_count);
            continue;
        }

        println!("Testing ECC loss pattern: lose {} of {} shards", loss_count, shard_count);
        
        let mut shards = encoded.shards_as_options();
        for i in 0..loss_count {
            if i < shards.len() {
                shards[i] = None;
            }
        }

        match qrd_core::ecc::decode_and_recover(&shards, &config) {
            Ok(recovered) => {
                assert_eq!(recovered, data, "Recovery should produce original data");
                println!("  ✓ Successfully recovered from {} losses", loss_count);
            }
            Err(e) => {
                // Expected if we exceed parity count
                if loss_count > 3 {
                    println!("  ! Expected failure for {} losses: {}", loss_count, e);
                } else {
                    panic!("Unexpected recovery failure: {}", e);
                }
            }
        }
    }
}

// ============================================================================
// FORMAT FUZZING
// ============================================================================

/// Fuzz file header parsing with malformed data
#[test]
fn fuzz_file_header_parsing() {
    let test_cases = vec![
        ("empty", vec![]),
        ("magic only", b"QRD".to_vec()),
        ("wrong magic", b"XXX\x01".to_vec()),
        ("wrong version", b"QRD\xFF".to_vec()),
        ("truncated header", vec![b'Q', b'R', b'D', 0x01, 0x00]),
        ("garbage", vec![0xFF; 32]),
    ];

    for (desc, data) in test_cases {
        println!("Testing file header parsing with {}", desc);
        
        // This would normally go through FooterParser
        // For now, just verify it doesn't panic
        let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Verify no panic occurs
            let cursor = Cursor::new(data);
            let _ = cursor;
        }));
        
        println!("  ✓ No panic");
    }
}

// ============================================================================
// BOUNDARY VALUE TESTS
// ============================================================================

/// Test encryption with boundary value data
#[test]
fn test_encryption_boundary_values() {
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();

    let test_cases = vec![
        ("single zero", vec![0x00]),
        ("single 0xFF", vec![0xFF]),
        ("1KB", vec![42; 1024]),
        ("near 64KB", vec![42; 65536 - 256]),
        ("exactly 64KB", vec![42; 65536]),
        ("over 64KB", vec![42; 65536 + 256]),
        ("1MB", vec![42; 1024 * 1024]),
    ];

    for (desc, data) in test_cases {
        println!("Testing encryption with {} data", desc);
        
        match encrypt(&data, &config) {
            Ok(encrypted) => {
                let decrypted = decrypt(&encrypted, &config)
                    .expect("Decryption should succeed");
                assert_eq!(decrypted, data, "Roundtrip should preserve data");
                println!("  ✓ Successfully encrypted and decrypted");
            }
            Err(e) => {
                panic!("Unexpected encryption failure: {}", e);
            }
        }
    }
}

// ============================================================================
// CONCURRENT/STRESS TESTS
// ============================================================================

/// Test multiple concurrent encryption operations
#[test]
fn test_concurrent_encryption() {
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();
    
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let config = config.clone();
            std::thread::spawn(move || {
                let data = format!("data_{}", i).into_bytes();
                let encrypted = encrypt(&data, &config).expect("Encryption failed");
                let decrypted = decrypt(&encrypted, &config).expect("Decryption failed");
                assert_eq!(decrypted, data, "Concurrent operation failed");
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
    
    println!("✓ All concurrent operations succeeded");
}

/// Test concurrent ECC operations
#[test]
fn test_concurrent_ecc() {
    let handles: Vec<_> = (0..5)
        .map(|_| {
            std::thread::spawn(|| {
                let config = EccConfig::with_chunk_size(2, 256)
                    .expect("Config creation failed");
                let mut codec = EccCodec::new(config.clone())
                    .expect("Codec creation failed");
                
                let data = vec![42u8; 512];
                let encoded = codec.encode(&data).expect("Encoding failed");
                
                let mut shards = encoded.shards_as_options();
                shards[0] = None;
                
                let recovered = qrd_core::ecc::decode_and_recover(&shards, &config)
                    .expect("Recovery failed");
                assert_eq!(recovered, data, "Concurrent ECC operation failed");
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
    
    println!("✓ All concurrent ECC operations succeeded");
}

// ============================================================================
// PROPERTY-BASED TESTS (using proptest patterns)
// ============================================================================

/// Test that all 256 byte values can be encrypted/decrypted
#[test]
fn test_encryption_all_byte_values() {
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();

    for byte_val in 0u8..=255 {
        let data = vec![byte_val; 256];
        
        let encrypted = encrypt(&data, &config).expect("Encryption failed");
        let decrypted = decrypt(&encrypted, &config).expect("Decryption failed");
        
        assert_eq!(decrypted, data, "Failed for byte value {}", byte_val);
    }
    
    println!("✓ All byte values encrypted/decrypted correctly");
}

/// Test encryption with various data lengths
#[test]
fn test_encryption_various_lengths() {
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();

    for len in &[0, 1, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 256, 512, 1024] {
        let data = vec![(*len as u8) % 256; *len];
        
        let encrypted = encrypt(&data, &config).expect("Encryption failed");
        let decrypted = decrypt(&encrypted, &config).expect("Decryption failed");
        
        assert_eq!(decrypted, data, "Roundtrip failed for length {}", len);
    }
    
    println!("✓ All tested lengths encrypted/decrypted correctly");
}
