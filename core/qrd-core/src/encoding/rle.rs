//! RLE (Run-Length Encoding) implementation

use crate::error::Result;
use super::Encoder;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

/// RLE encoder for repetitive data
pub struct RleEncoder;

impl RleEncoder {
    /// Create new RLE encoder
    pub fn new() -> Self {
        RleEncoder
    }
}

impl Default for RleEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder for RleEncoder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();
        let mut i = 0;

        while i < data.len() {
            let current_byte = data[i];
            let mut run_length = 1usize;

            // Count consecutive identical bytes
            while i + run_length < data.len() && data[i + run_length] == current_byte {
                run_length += 1;
            }

            // Write run_length (4 bytes, little-endian)
            result.write_u32::<LittleEndian>(run_length as u32)?;
            // Write value (1 byte)
            result.push(current_byte);

            i += run_length;
        }

        Ok(result)
    }

    fn decode(&self, data: &[u8], _expected_length: usize) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        let mut cursor = Cursor::new(data);

        while cursor.position() < data.len() as u64 {
            let run_length = cursor.read_u32::<LittleEndian>()?;

            if cursor.position() >= data.len() as u64 {
                break;
            }

            let byte = cursor.read_u8()?;

            for _ in 0..run_length {
                result.push(byte);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_encode_decode_simple() {
        let encoder = RleEncoder;
        let data = vec![1u8, 1u8, 1u8, 2u8, 2u8];
        let encoded = encoder.encode(&data).unwrap();
        let decoded = encoder.decode(&encoded, data.len()).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_rle_single_values() {
        let encoder = RleEncoder;
        let data = vec![1u8, 2u8, 3u8, 4u8];
        let encoded = encoder.encode(&data).unwrap();
        let decoded = encoder.decode(&encoded, data.len()).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_rle_all_same() {
        let encoder = RleEncoder;
        let data = vec![5u8; 100];
        let encoded = encoder.encode(&data).unwrap();
        let decoded = encoder.decode(&encoded, data.len()).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_rle_empty() {
        let encoder = RleEncoder;
        let data = vec![];
        let encoded = encoder.encode(&data).unwrap();
        assert!(encoded.is_empty());
    }
}
