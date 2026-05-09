//! Integration test for Phase 2 advanced features

use qrd_core::ecc::{EccCodec, EccConfig};
use qrd_core::encryption::{EncryptionConfig, encrypt, decrypt};
use qrd_core::utils::simd::SimdOps;
use qrd_core::utils::bit_ops::*;
use qrd_core::encoding::{PlainEncoder, PlainDecoder};

#[test]
fn test_encryption_integration() {
    let key = EncryptionConfig::generate_key();
    let config = EncryptionConfig::new(key).unwrap();

    let original_data = b"Hello, QRD with encryption!";

    // Encrypt
    let encrypted = encrypt(original_data, &config).unwrap();
    assert_ne!(encrypted, original_data);

    // Decrypt
    let decrypted = decrypt(&encrypted, &config).unwrap();
    assert_eq!(decrypted, original_data);
}

#[test]
fn test_ecc_integration() {
    let config = EccConfig::with_chunk_size(2, 1024).unwrap();
    let mut codec = EccCodec::new(config).unwrap();

    let original_data = vec![42u8; 2048];

    // Encode with ECC
    let encoded = codec.encode(&original_data).unwrap();

    // Simulate data loss (corrupt some data chunks)
    let mut shards = encoded.shards_as_options();
    shards[0] = None; // Lose first data chunk

    // Recover
    let recovered = qrd_core::ecc::decode_and_recover(&shards, &config).unwrap();
    assert_eq!(recovered, original_data);
}

#[test]
fn test_simd_operations() {
    let ops = SimdOps::new();

    let data = vec![1u8; 1000];
    let mut dst = vec![0u8; 1000];

    // Test memcpy
    ops.memcpy(&mut dst, &data).unwrap();
    assert_eq!(dst, data);

    // Test XOR
    let xor_data = vec![255u8; 1000];
    ops.xor(&mut dst, &xor_data).unwrap();
    assert_eq!(dst, vec![254u8; 1000]);

    // Test count_bytes
    let count = ops.count_bytes(&data, 1);
    assert_eq!(count, 1000);

    // Test delta encoding
    let delta_data: Vec<i32> = (0..100).map(|i| i * 2).collect();
    let encoded = ops.delta_encode_i32(&delta_data).unwrap();
    let decoded = ops.delta_decode_i32(&encoded).unwrap();
    assert_eq!(decoded, delta_data);
}

#[test]
fn test_bit_operations() {
    // Test bit packing/unpacking
    let values: Vec<u32> = vec![1, 2, 3, 4, 5];
    let packed = pack_bits(&values, 3).unwrap();
    let unpacked = unpack_bits(&packed, 3, values.len()).unwrap();
    assert_eq!(unpacked, values);

    // Test bit scanning
    let data = vec![0b10101010u8, 0b01010101u8];
    let positions = scan_bits(&data, true);
    assert_eq!(positions, vec![1, 3, 5, 7, 8, 10, 12, 14]);

    // Test population count
    let count = popcount(&data);
    assert_eq!(count, 8);
}

#[test]
fn test_encoding_with_simd() {
    let ops = SimdOps::new();

    // Test with plain encoding
    let original: Vec<i32> = (0..1000).collect();
    let mut encoder = PlainEncoder::new();
    let mut encoded = Vec::new();
    encoder.encode(&original, &mut encoded).unwrap();

    let mut decoder = PlainDecoder::new();
    let mut decoded = Vec::new();
    decoder.decode(&encoded, &mut decoded).unwrap();

    assert_eq!(decoded, original);

    // Test SIMD delta encoding on the data
    let delta_encoded = ops.delta_encode_i32(&original).unwrap();
    let delta_decoded = ops.delta_decode_i32(&delta_encoded).unwrap();
    assert_eq!(delta_decoded, original);
}

#[test]
fn test_combined_features() {
    // Test encryption + ECC + SIMD operations together
    let key = EncryptionConfig::generate_key();
    let encrypt_config = EncryptionConfig::new(key).unwrap();
    let ecc_config = EccConfig::with_chunk_size(2, 512).unwrap();
    let mut ecc_codec = EccCodec::new(ecc_config).unwrap();
    let ops = SimdOps::new();

    let original_data = vec![42u8; 1024];

    // Encrypt
    let encrypted = encrypt(&original_data, &encrypt_config).unwrap();

    // Add ECC
    let ecc_encoded = ecc_codec.encode(&encrypted).unwrap();

    // Simulate corruption
    let mut shards = ecc_encoded.shards_as_options();
    shards[1] = None; // Lose a chunk

    // Recover with ECC
    let ecc_recovered = qrd_core::ecc::decode_and_recover(&shards, &ecc_config).unwrap();

    // Decrypt
    let final_decrypted = decrypt(&ecc_recovered, &encrypt_config).unwrap();

    assert_eq!(final_decrypted, original_data);

    // Test SIMD operations on the result
    let mut test_data = vec![0u8; 1024];
    ops.memcpy(&mut test_data, &final_decrypted).unwrap();
    assert_eq!(test_data, original_data);
}