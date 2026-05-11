//! Utility-focused tests

use qrd_core::utils::bit_ops::BitOps;
use qrd_core::utils::simd::SimdOps;

#[test]
fn test_bitops_reverse_bits() {
    assert_eq!(BitOps::reverse_bits_byte(0b00001111), 0b11110000);
    assert_eq!(BitOps::reverse_bits_byte(0b10101010), 0b01010101);
}

#[test]
fn test_popcount_various() {
    assert_eq!(BitOps::popcount(0), 0);
    assert_eq!(BitOps::popcount(1), 1);
    assert_eq!(BitOps::popcount(0xFFFF_FFFF), 32);
}

#[test]
fn test_simd_memcpy_and_xor() {
    let ops = SimdOps::new();
    let data = vec![1u8; 128];
    let mut dst = vec![0u8; 128];
    ops.memcpy(&mut dst, &data).unwrap();
    assert_eq!(dst, data);

    let xor_with = vec![0xFFu8; 128];
    ops.xor(&mut dst, &xor_with).unwrap();
    for b in dst.iter() {
        assert_eq!(*b, 0xFE);
    }
}

#[test]
fn test_simd_count_bytes() {
    let ops = SimdOps::new();
    let data = vec![1u8; 1000];
    let count = ops.count_bytes(&data, 1);
    assert_eq!(count, 1000);
}
