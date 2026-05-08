//! Encoding algorithms for columnar data

use crate::error::{Error, Result};
use std::fmt;

pub mod plain;
pub mod rle;

/// Encoding type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncodingType {
    /// Raw unencoded data
    Plain,
    /// Run-length encoding
    Rle,
    /// Bit-packed encoding
    BitPacked,
    /// Delta-of-deltas for sorted integers
    DeltaBinary,
    /// Delta encoding for byte arrays
    DeltaByteArray,
    /// Byte-stream split (floating point)
    ByteStreamSplit,
    /// Dictionary with RLE
    DictionaryRle,
    /// No encoding (pre-encoded)
    Passthrough,
}

impl fmt::Display for EncodingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodingType::Plain => write!(f, "PLAIN"),
            EncodingType::Rle => write!(f, "RLE"),
            EncodingType::BitPacked => write!(f, "BIT_PACKED"),
            EncodingType::DeltaBinary => write!(f, "DELTA_BINARY"),
            EncodingType::DeltaByteArray => write!(f, "DELTA_BYTE_ARRAY"),
            EncodingType::ByteStreamSplit => write!(f, "BYTE_STREAM_SPLIT"),
            EncodingType::DictionaryRle => write!(f, "DICTIONARY_RLE"),
            EncodingType::Passthrough => write!(f, "PASSTHROUGH"),
        }
    }
}

impl EncodingType {
    /// Convert to byte ID for binary format
    pub fn to_id(self) -> u8 {
        match self {
            EncodingType::Plain => 0,
            EncodingType::Rle => 1,
            EncodingType::BitPacked => 2,
            EncodingType::DeltaBinary => 3,
            EncodingType::DeltaByteArray => 4,
            EncodingType::ByteStreamSplit => 5,
            EncodingType::DictionaryRle => 6,
            EncodingType::Passthrough => 7,
        }
    }

    /// Convert from byte ID
    pub fn from_id(id: u8) -> Result<Self> {
        match id {
            0 => Ok(EncodingType::Plain),
            1 => Ok(EncodingType::Rle),
            2 => Ok(EncodingType::BitPacked),
            3 => Ok(EncodingType::DeltaBinary),
            4 => Ok(EncodingType::DeltaByteArray),
            5 => Ok(EncodingType::ByteStreamSplit),
            6 => Ok(EncodingType::DictionaryRle),
            7 => Ok(EncodingType::Passthrough),
            _ => Err(Error::DecodingError(format!("Unknown encoding ID: {}", id))),
        }
    }
}

/// Generic encoder trait
pub trait Encoder: Send + Sync {
    /// Encode data
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Decode data
    fn decode(&self, data: &[u8], expected_length: usize) -> Result<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_round_trip() {
        for id in 0..8 {
            let encoding = EncodingType::from_id(id).unwrap();
            assert_eq!(encoding.to_id(), id);
        }
    }
}
