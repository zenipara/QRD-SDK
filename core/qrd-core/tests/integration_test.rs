//! Integration test for Phase 2 advanced features

use qrd_core::ecc::{EccCodec, EccConfig};
use qrd_core::encryption::{decrypt, encrypt, EncryptionConfig};
use qrd_core::utils::simd::SimdOps;

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
    let mut codec = EccCodec::new(config.clone()).unwrap();

    let original_data = vec![42u8; 2048];

    // Encode with ECC
    let encoded = codec.encode(&original_data).unwrap();

    // Simulate data loss (corrupt some data chunks)
    let mut shards = encoded.shards_as_options();
    shards[0] = None; // Lose first data chunk

    // Recover
    let recovered = qrd_core::ecc::decode_and_recover_with_options(&encoded, &shards).unwrap();
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

// Re-enabled: BitOps functions now properly available
#[test]
fn test_bit_operations() {
    use qrd_core::utils::bit_ops::BitOps;

    // Test bitscan operations
    // Test trailing/leading zeros
    assert_eq!(BitOps::trailing_zeros(0b10100000), 5);
    assert_eq!(BitOps::leading_zeros(0b00000111), 61);

    // Test population count on individual bytes represented as u64
    let count1 = BitOps::popcount(0b10101010u64);
    assert_eq!(count1, 4, "0b10101010 should have 4 set bits");

    let count2 = BitOps::popcount(0b01010101u64);
    assert_eq!(count2, 4, "0b01010101 should have 4 set bits");

    // Test popcount across both bytes
    let total_count = count1 + count2;
    assert_eq!(total_count, 8, "Should count 8 set bits total");

    // Test bitscan forward (find first set bit)
    assert_eq!(BitOps::bitscan_forward(0b10101010), Some(1));
    assert_eq!(BitOps::bitscan_forward(0), None);
    assert_eq!(BitOps::bitscan_forward(0b00001000), Some(3));

    // Test bitscan reverse (find last set bit)
    assert_eq!(BitOps::bitscan_reverse(0b10101010), Some(7));
    assert_eq!(BitOps::bitscan_reverse(0), None);
    assert_eq!(BitOps::bitscan_reverse(0b00001000), Some(3));

    // Test min_bits_for_value
    assert_eq!(BitOps::min_bits_for_value(0), 1);
    assert_eq!(BitOps::min_bits_for_value(7), 3); // 0-7 needs 3 bits
    assert_eq!(BitOps::min_bits_for_value(15), 4); // 0-15 needs 4 bits
}

#[test]
fn test_encoding_with_simd() {
    use qrd_core::encoding::plain::PlainEncoder;
    use qrd_core::encoding::Encoder;

    let ops = SimdOps::new();

    // Test with plain encoding
    let original: Vec<i32> = (0..1000).collect();
    let encoder = PlainEncoder::new();
    let encoded = encoder
        .encode(
            &original
                .iter()
                .flat_map(|&x| x.to_le_bytes())
                .collect::<Vec<_>>(),
        )
        .unwrap();

    let decoded_bytes = encoder.decode(&encoded, original.len() * 4).unwrap();
    let mut decoded: Vec<i32> = Vec::new();
    for chunk in decoded_bytes.chunks(4) {
        if chunk.len() == 4 {
            decoded.push(i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
    }

    assert_eq!(
        decoded, original,
        "Plain encoder should preserve i32 values"
    );

    // Test SIMD delta encoding on the data
    let delta_encoded = ops.delta_encode_i32(&original).unwrap();
    let delta_decoded = ops.delta_decode_i32(&delta_encoded).unwrap();
    assert_eq!(
        delta_decoded, original,
        "SIMD delta encode/decode should be reversible"
    );
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
    let ecc_recovered =
        qrd_core::ecc::decode_and_recover_with_options(&ecc_encoded, &shards).unwrap();

    // Decrypt
    let final_decrypted = decrypt(&ecc_recovered, &encrypt_config).unwrap();

    assert_eq!(final_decrypted, original_data);

    // Test SIMD operations on the result
    let mut test_data = vec![0u8; 1024];
    ops.memcpy(&mut test_data, &final_decrypted).unwrap();
    assert_eq!(test_data, original_data);
}
