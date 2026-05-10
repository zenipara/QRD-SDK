//! BIT_PACKED encoding implementation

use crate::encoding::Encoder;
use crate::error::{Error, Result};

/// BIT_PACKED encoder for boolean data
pub struct BitPackedEncoder;

impl BitPackedEncoder {
    /// Create new encoder
    pub fn new() -> Self {
        BitPackedEncoder
    }
}

impl Encoder for BitPackedEncoder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        // For now, assume data is already boolean bytes (0 or 1)
        // In practice, this should pack 8 booleans per byte
        if data.iter().all(|&b| b == 0 || b == 1) {
            let mut packed = Vec::with_capacity((data.len() + 7) / 8);
            for chunk in data.chunks(8) {
                let mut byte = 0u8;
                for (i, &bit) in chunk.iter().enumerate() {
                    if bit == 1 {
                        byte |= 1 << (7 - i);
                    }
                }
                packed.push(byte);
            }
            Ok(packed)
        } else {
            Err(Error::EncodingError(
                "BIT_PACKED requires boolean data (0/1 bytes)".to_string(),
            ))
        }
    }

    fn decode(&self, data: &[u8], expected_length: usize) -> Result<Vec<u8>> {
        let mut unpacked = Vec::with_capacity(expected_length);
        for &byte in data {
            for i in 0..8 {
                if unpacked.len() >= expected_length {
                    break;
                }
                let bit = if (byte & (1 << (7 - i))) != 0 { 1 } else { 0 };
                unpacked.push(bit);
            }
        }
        Ok(unpacked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_packed_basic() {
        let encoder = BitPackedEncoder::new();
        let data = vec![1, 0, 1, 0, 1, 0, 1, 0];
        let encoded = encoder.encode(&data).unwrap();
        assert_eq!(encoded, vec![0b10101010]);

        let decoded = encoder.decode(&encoded, 8).unwrap();
        assert_eq!(decoded, data);
    }
}
