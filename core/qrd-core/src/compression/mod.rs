//! Compression codecs with adaptive selection

pub mod entropy;

pub use entropy::{EntropyCalculator, CompressionSelector};

use std::fmt;

/// Compression codec
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionCodec {
    /// No compression
    None,
    /// Zstandard compression
    Zstd,
    /// LZ4 compression
    Lz4,
}

impl fmt::Display for CompressionCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionCodec::None => write!(f, "NONE"),
            CompressionCodec::Zstd => write!(f, "ZSTD"),
            CompressionCodec::Lz4 => write!(f, "LZ4"),
        }
    }
}

impl CompressionCodec {
    /// Convert to byte ID for binary format
    pub fn to_id(self) -> u8 {
        match self {
            CompressionCodec::None => 0,
            CompressionCodec::Zstd => 1,
            CompressionCodec::Lz4 => 2,
        }
    }

    /// Convert from byte ID
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(CompressionCodec::None),
            1 => Some(CompressionCodec::Zstd),
            2 => Some(CompressionCodec::Lz4),
            _ => None,
        }
    }
}

/// Compression level (0-10)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionLevel(u8);

impl CompressionLevel {
    /// Create a compression level
    pub fn new(level: u8) -> Self {
        CompressionLevel(level.min(10))
    }

    /// Get the level as u8
    pub fn level(self) -> u8 {
        self.0
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        CompressionLevel(3)
    }
}

/// Compress data
pub fn compress(
    data: &[u8],
    codec: CompressionCodec,
    level: CompressionLevel,
) -> crate::error::Result<Vec<u8>> {
    match codec {
        CompressionCodec::None => Ok(data.to_vec()),
        CompressionCodec::Zstd => compress_zstd(data, level),
        CompressionCodec::Lz4 => compress_lz4(data, level),
    }
}

/// Decompress data
pub fn decompress(
    data: &[u8],
    codec: CompressionCodec,
) -> crate::error::Result<Vec<u8>> {
    match codec {
        CompressionCodec::None => Ok(data.to_vec()),
        CompressionCodec::Zstd => decompress_zstd(data),
        CompressionCodec::Lz4 => decompress_lz4(data),
    }
}

fn compress_zstd(data: &[u8], level: CompressionLevel) -> crate::error::Result<Vec<u8>> {
    zstd::encode_all(data, level.0 as i32).map_err(|e| {
        crate::error::Error::CompressionError(format!("ZSTD compression failed: {}", e))
    })
}

fn decompress_zstd(data: &[u8]) -> crate::error::Result<Vec<u8>> {
    zstd::decode_all(data).map_err(|e| {
        crate::error::Error::DecompressionError(format!("ZSTD decompression failed: {}", e))
    })
}

fn compress_lz4(data: &[u8], _level: CompressionLevel) -> crate::error::Result<Vec<u8>> {
    lz4::block::compress(data, Some(lz4::block::CompressionMode::FAST(0)), true).map_err(|e| {
        crate::error::Error::CompressionError(format!("LZ4 compression failed: {}", e))
    })
}

fn decompress_lz4(data: &[u8]) -> crate::error::Result<Vec<u8>> {
    lz4::block::decompress(data, None).map_err(|e| {
        crate::error::Error::DecompressionError(format!("LZ4 decompression failed: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_round_trip() {
        for id in 0..3 {
            if let Some(codec) = CompressionCodec::from_id(id) {
                assert_eq!(codec.to_id(), id);
            }
        }
    }

    #[test]
    fn test_compress_decompress_zstd() {
        let data = b"hello world hello world hello world";
        let compressed = compress_zstd(data, CompressionLevel::default()).unwrap();
        let decompressed = decompress_zstd(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compress_decompress_lz4() {
        let data = b"hello world hello world hello world";
        let compressed = compress_lz4(data, CompressionLevel::default()).unwrap();
        let decompressed = decompress_lz4(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }
}
