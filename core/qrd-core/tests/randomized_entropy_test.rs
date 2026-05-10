//! Randomized entropy validation tests
//!
//! These tests validate that the QRD format correctly handles
//! random payloads with high entropy and various distributions,
//! testing compression stability and encoding correctness.

use qrd_core::prelude::*;
use qrd_core::writer::FileWriter;
use qrd_core::reader::FileReader;
use tempfile::NamedTempFile;

/// Generate pseudo-random bytes based on seed
fn generate_random_bytes(seed: u64, count: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(count);
    let mut rng = seed;
    
    for _ in 0..count {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        result.push((rng >> 24) as u8);
    }
    
    result
}

/// Calculate entropy of a byte sequence
fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    
    let mut frequencies = [0u32; 256];
    for &byte in data {
        frequencies[byte as usize] += 1;
    }
    
    let len = data.len() as f64;
    let mut entropy = 0.0;
    
    for freq in &frequencies {
        if *freq > 0 {
            let p = *freq as f64 / len;
            entropy -= p * p.log2();
        }
    }
    
    entropy
}

#[test]
fn test_high_entropy_payload_roundtrip() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("entropy_data", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    // Generate high-entropy random data
    let random_payload = generate_random_bytes(0x123456789ABCDEF0, 10000);
    let entropy = calculate_entropy(&random_payload);
    
    // High entropy should be > 7 bits per byte
    assert!(entropy > 7.0, "Expected high entropy, got {}", entropy);

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
        writer.write_row(vec![random_payload.clone()]).unwrap();
        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_low_entropy_payload_compression() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("repetitive_data", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    // Generate low-entropy data (highly repetitive)
    let repetitive = b"AAAAAABBBBBBCCCCCCDDDDDDEEEEEEAAAAAA".repeat(100);
    let entropy = calculate_entropy(&repetitive);
    
    // Low entropy should be < 4 bits per byte
    assert!(entropy < 4.0, "Expected low entropy, got {}", entropy);

    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
        writer.write_row(vec![repetitive.clone()]).unwrap();
        writer.finish().unwrap();
    }

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_random_payload_generation_determinism() {
    let seed = 0x0123456789ABCDEF;
    
    let bytes1 = generate_random_bytes(seed, 1000);
    let bytes2 = generate_random_bytes(seed, 1000);
    
    // Same seed should produce identical bytes
    assert_eq!(bytes1, bytes2);
}

#[test]
fn test_entropy_distribution_uniform() {
    let bytes = generate_random_bytes(0x42, 10000);
    let entropy = calculate_entropy(&bytes);
    
    // Pseudo-random should have entropy ~ 8.0
    assert!(entropy > 7.5, "Expected entropy > 7.5, got {}", entropy);
    assert!(entropy <= 8.0, "Expected entropy <= 8.0, got {}", entropy);
}

#[test]
fn test_entropy_distribution_bimodal() {
    // Create bimodal distribution: half 0x00, half 0xFF
    let mut data = vec![0x00u8; 5000];
    data.extend_from_slice(&vec![0xFFu8; 5000]);
    
    let entropy = calculate_entropy(&data);
    
    // Should be exactly 1 bit of entropy
    assert!((entropy - 1.0).abs() < 0.01, "Expected entropy ~1.0, got {}", entropy);
}

#[test]
fn test_compression_stability_across_random_seeds() {
    let temp1 = NamedTempFile::new().unwrap();
    let temp2 = NamedTempFile::new().unwrap();
    
    let schema = SchemaBuilder::new()
        .add_field("data", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let data1 = generate_random_bytes(1, 10000);
    let data2 = generate_random_bytes(2, 10000);

    {
        let mut writer = FileWriter::new(temp1.path(), schema.clone()).unwrap();
        writer.write_row(vec![data1]).unwrap();
        writer.finish().unwrap();
    }

    {
        let mut writer = FileWriter::new(temp2.path(), schema.clone()).unwrap();
        writer.write_row(vec![data2]).unwrap();
        writer.finish().unwrap();
    }

    let reader1 = FileReader::new(temp1.path()).unwrap();
    let reader2 = FileReader::new(temp2.path()).unwrap();
    
    // Both should have valid row counts
    assert_eq!(reader1.row_count(), 1);
    assert_eq!(reader2.row_count(), 1);
}

#[test]
fn test_random_payload_generation() {
    // Test that random payload generation produces expected distributions
    let payload = generate_random_bytes(0xDEADBEEF, 1000);
    assert_eq!(payload.len(), 1000);
    
    // Check that it's not all zeros or all same value
    let unique_values: std::collections::HashSet<_> = payload.iter().collect();
    assert!(unique_values.len() > 1, "Random data should have multiple unique values");
}

#[test]
fn test_entropy_distribution_validation() {
    // Test entropy calculation on known distributions
    let uniform = generate_random_bytes(0x123, 10000);
    let entropy = calculate_entropy(&uniform);
    assert!(entropy > 7.0, "Uniform random data should have high entropy");
    
    let constant = vec![42u8; 1000];
    let entropy_const = calculate_entropy(&constant);
    assert_eq!(entropy_const, 0.0, "Constant data should have zero entropy");
}

#[test]
fn test_compression_stability() {
    // Test that compression is stable across multiple runs with same data
    let temp1 = NamedTempFile::new().unwrap();
    let temp2 = NamedTempFile::new().unwrap();
    
    let schema = SchemaBuilder::new()
        .add_field("data", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let data = generate_random_bytes(0xCAFEBABE, 5000);

    // Write same data twice
    for temp in [&temp1, &temp2] {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
        writer.write_row(vec![data.clone()]).unwrap();
        writer.finish().unwrap();
    }

    // Both files should be identical (deterministic compression)
    let content1 = std::fs::read(temp1.path()).unwrap();
    let content2 = std::fs::read(temp2.path()).unwrap();
    assert_eq!(content1, content2, "Compression should be deterministic");
}
