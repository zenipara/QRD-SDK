//! PLAIN encoding - raw data without transformation

use crate::error::Result;
use super::Encoder;

/// Plain encoder - passes data through unchanged
pub struct PlainEncoder;

impl PlainEncoder {
    /// Create new plain encoder
    pub fn new() -> Self {
        PlainEncoder
    }
}

impl Default for PlainEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder for PlainEncoder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Plain encoding is just the raw data
        Ok(data.to_vec())
    }

    fn decode(&self, data: &[u8], _expected_length: usize) -> Result<Vec<u8>> {
        // Plain decoding is just the raw data
        Ok(data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_encode_decode() {
        let encoder = PlainEncoder;
        let data = b"hello world";
        let encoded = encoder.encode(data).unwrap();
        assert_eq!(encoded, data);

        let decoded = encoder.decode(&encoded, data.len()).unwrap();
        assert_eq!(decoded, data);
    }
}
