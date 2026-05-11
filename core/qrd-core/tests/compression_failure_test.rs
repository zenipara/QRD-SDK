//! Decompression failure handling and error path tests
//!
//! Tests various failure scenarios in compression/decompression
//! to improve coverage of error handling paths

use qrd_core::compression::{CompressionCodec, CompressionLevel, compress, decompress};
use qrd_core::error::Result;

/// Test decompression of corrupted zstd data
#[test]
fn test_decompress_corrupted_zstd() {
    // Invalid ZSTD compressed data
    let corrupted = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x00];
    let result = decompress(&corrupted, CompressionCodec::Zstd);
    assert!(result.is_err());
}

/// Test decompression of corrupted lz4 data
#[test]
fn test_decompress_corrupted_lz4() {
    // Invalid LZ4 compressed data
    let corrupted = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x00];
    let result = decompress(&corrupted, CompressionCodec::Lz4);
    assert!(result.is_err());
}

/// Test decompression of partial/truncated data
#[test]
fn test_decompress_truncated_zstd() {
    let data = b"this is test data for compression";
    let compressed = compress(data, CompressionCodec::Zstd, CompressionLevel::default())
        .unwrap();
    
    // Truncate the compressed data
    let truncated = &compressed[0..compressed.len().saturating_sub(5)];
    let result = decompress(truncated, CompressionCodec::Zstd);
    
    // Should fail gracefully
    assert!(result.is_err());
}

/// Test decompression of truncated lz4 data
#[test]
fn test_decompress_truncated_lz4() {
    let data = b"this is test data for compression";
    let compressed = compress(data, CompressionCodec::Lz4, CompressionLevel::default())
        .unwrap();
    
    // Truncate the compressed data
    if compressed.len() > 5 {
        let truncated = &compressed[0..compressed.len() - 5];
        let result = decompress(truncated, CompressionCodec::Lz4);
        assert!(result.is_err());
    }
}

/// Test decompression with invalid codec ID
#[test]
fn test_decompress_invalid_codec_id() {
    let data = vec![0x01, 0x02, 0x03];
    
    // CompressionCodec::from_id() should return None for invalid IDs
    let invalid_ids = vec![3u8, 4, 5, 255];
    
    for id in invalid_ids {
        match CompressionCodec::from_id(id) {
            Some(_) => panic!("Should not recognize codec ID {}", id),
            None => {
                // Expected behavior - cannot decompress with invalid codec
            }
        }
    }
}

/// Test compression with empty data
#[test]
fn test_compress_empty_data() {
    let data = b"";
    
    for codec in &[CompressionCodec::None, CompressionCodec::Zstd, CompressionCodec::Lz4] {
        let result = compress(data, *codec, CompressionLevel::default());
        assert!(result.is_ok());
        let compressed = result.unwrap();
        
        let decompressed = decompress(&compressed, *codec).unwrap();
        assert_eq!(decompressed, data);
    }
}

/// Test compression roundtrip with various compression levels
#[test]
fn test_compress_all_levels() {
    let data = b"The quick brown fox jumps over the lazy dog. ".repeat(20);
    
    for level in 0..=10 {
        let comp_level = CompressionLevel::new(level);
        let compressed = compress(&data, CompressionCodec::Zstd, comp_level).unwrap();
        let decompressed = decompress(&compressed, CompressionCodec::Zstd).unwrap();
        assert_eq!(decompressed, data);
    }
}

/// Test lz4 compression levels (only FAST mode)
#[test]
fn test_lz4_compression_level() {
    let data = b"Test data for LZ4 compression with different levels";
    
    for level in 0..=10 {
        let comp_level = CompressionLevel::new(level);
        let compressed = compress(data, CompressionCodec::Lz4, comp_level).unwrap();
        let decompressed = decompress(&compressed, CompressionCodec::Lz4).unwrap();
        assert_eq!(decompressed, data);
    }
}

/// Test compression with very large data
#[test]
fn test_compress_very_large_data() {
    let large_data = vec![0xAB; 100 * 1024 * 1024];  // 100MB of data
    
    for codec in &[CompressionCodec::None, CompressionCodec::Zstd, CompressionCodec::Lz4] {
        let result = compress(&large_data, *codec, CompressionLevel::default());
        
        // Should not crash or OOM
        if let Ok(compressed) = result {
            let decompressed = decompress(&compressed, *codec).unwrap();
            assert_eq!(decompressed, large_data);
        }
    }
}

/// Test decompression with None codec
#[test]
fn test_decompress_none_codec() {
    let data = b"data that is not compressed";
    let result = decompress(data, CompressionCodec::None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), data);
}

/// Test compression with None codec (pass-through)
#[test]
fn test_compress_none_codec() {
    let data = b"data that will not be compressed";
    let result = compress(data, CompressionCodec::None, CompressionLevel::default());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), data);
}

/// Test codec round-trip conversions
#[test]
fn test_codec_id_roundtrip() {
    let codecs = vec![
        CompressionCodec::None,
        CompressionCodec::Zstd,
        CompressionCodec::Lz4,
    ];
    
    for codec in codecs {
        let id = codec.to_id();
        let roundtrip = CompressionCodec::from_id(id);
        assert_eq!(roundtrip, Some(codec));
    }
}

/// Test compression of highly repetitive data
#[test]
fn test_compress_repetitive_data() {
    let repetitive = vec![0x42; 10 * 1024 * 1024];  // 10MB of same byte
    
    for codec in &[CompressionCodec::Zstd, CompressionCodec::Lz4] {
        let compressed = compress(&repetitive, *codec, CompressionLevel::default()).unwrap();
        
        // Repetitive data should compress well
        let compression_ratio = compressed.len() as f64 / repetitive.len() as f64;
        assert!(compression_ratio < 0.5, "Repetitive data should compress to <50%");
        
        let decompressed = decompress(&compressed, *codec).unwrap();
        assert_eq!(decompressed, repetitive);
    }
}

/// Test compression of random data
#[test]
fn test_compress_random_data() {
    // Create pseudo-random data
    let mut data = Vec::new();
    let mut seed = 42u32;
    for _ in 0..1000 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        data.push((seed >> 8) as u8);
    }
    
    for codec in &[CompressionCodec::Zstd, CompressionCodec::Lz4] {
        let compressed = compress(&data, *codec, CompressionLevel::default()).unwrap();
        let decompressed = decompress(&compressed, *codec).unwrap();
        assert_eq!(decompressed, data);
    }
}

/// Test decompression with mismatched codec
#[test]
fn test_decompress_mismatched_codec() {
    let data = b"test data for compression";
    
    // Compress with ZSTD
    let zstd_compressed = compress(data, CompressionCodec::Zstd, CompressionLevel::default())
        .unwrap();
    
    // Try to decompress with LZ4 - should fail
    let result = decompress(&zstd_compressed, CompressionCodec::Lz4);
    assert!(result.is_err());
}

/// Test compression level edge cases
#[test]
fn test_compression_level_edge_cases() {
    let data = b"test data for compression level testing";
    
    // Level 0 (minimum)
    let level_0 = CompressionLevel::new(0);
    let compressed_0 = compress(data, CompressionCodec::Zstd, level_0).unwrap();
    assert!(decompress(&compressed_0, CompressionCodec::Zstd).is_ok());
    
    // Level 10 (maximum)
    let level_10 = CompressionLevel::new(10);
    let compressed_10 = compress(data, CompressionCodec::Zstd, level_10).unwrap();
    assert!(decompress(&compressed_10, CompressionCodec::Zstd).is_ok());
    
    // Level > 10 (should clamp to 10)
    let level_20 = CompressionLevel::new(20);
    let compressed_20 = compress(data, CompressionCodec::Zstd, level_20).unwrap();
    assert!(decompress(&compressed_20, CompressionCodec::Zstd).is_ok());
}

/// Test decompression of data with null bytes
#[test]
fn test_compress_data_with_null_bytes() {
    let data = vec![0u8; 1000];  // All null bytes
    
    for codec in &[CompressionCodec::Zstd, CompressionCodec::Lz4] {
        let compressed = compress(&data, *codec, CompressionLevel::default()).unwrap();
        let decompressed = decompress(&compressed, *codec).unwrap();
        assert_eq!(decompressed, data);
    }
}

/// Test codec display formatting
#[test]
fn test_codec_display() {
    assert_eq!(format!("{}", CompressionCodec::None), "NONE");
    assert_eq!(format!("{}", CompressionCodec::Zstd), "ZSTD");
    assert_eq!(format!("{}", CompressionCodec::Lz4), "LZ4");
}
