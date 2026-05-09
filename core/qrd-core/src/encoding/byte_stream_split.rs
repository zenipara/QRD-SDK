//! BYTE_STREAM_SPLIT encoding implementation

use crate::encoding::Encoder;
use crate::error::{Error, Result};

/// BYTE_STREAM_SPLIT encoder for floating-point data
pub struct ByteStreamSplitEncoder;

impl ByteStreamSplitEncoder {
    /// Create new encoder
    pub fn new() -> Self {
        ByteStreamSplitEncoder
    }
}

impl Encoder for ByteStreamSplitEncoder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Assume data is sequence of fixed-size values (4 or 8 bytes)
        let value_size = if data.len() % 8 == 0 { 8 } else if data.len() % 4 == 0 { 4 } else {
            return Err(Error::EncodingError(
                "BYTE_STREAM_SPLIT requires data length to be multiple of 4 or 8 bytes".to_string(),
            ));
        };

        let num_values = data.len() / value_size;
        encode_byte_stream_split(data, value_size, num_values)
    }

    fn decode(&self, data: &[u8], expected_length: usize) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Assume data is sequence of fixed-size values (4 or 8 bytes)
        let value_size = if expected_length % 8 == 0 { 8 } else if expected_length % 4 == 0 { 4 } else {
            return Err(Error::DecodingError(
                "BYTE_STREAM_SPLIT requires expected length to be multiple of 4 or 8 bytes".to_string(),
            ));
        };

        let num_values = expected_length / value_size;
        decode_byte_stream_split(data, value_size, num_values)
    }
}

/// Encode using byte stream split
fn encode_byte_stream_split(data: &[u8], value_size: usize, num_values: usize) -> Result<Vec<u8>> {
    let mut streams: Vec<Vec<u8>> = vec![Vec::new(); value_size];

    for value_idx in 0..num_values {
        let value_start = value_idx * value_size;
        for byte_idx in 0..value_size {
            streams[byte_idx].push(data[value_start + byte_idx]);
        }
    }

    // Concatenate all streams
    let mut result = Vec::with_capacity(data.len());
    for stream in streams {
        result.extend(stream);
    }

    Ok(result)
}

/// Decode byte stream split
fn decode_byte_stream_split(data: &[u8], value_size: usize, num_values: usize) -> Result<Vec<u8>> {
    let expected_data_len = num_values * value_size;
    if data.len() != expected_data_len {
        return Err(Error::DecodingError(
            format!("Data length {} doesn't match expected {} for {} values of size {}",
                data.len(), expected_data_len, num_values, value_size)
        ));
    }

    let mut result = vec![0u8; expected_data_len];
    let stream_size = num_values;

    for value_idx in 0..num_values {
        for byte_idx in 0..value_size {
            let stream_offset = byte_idx * stream_size + value_idx;
            let value_offset = value_idx * value_size + byte_idx;
            result[value_offset] = data[stream_offset];
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_stream_split_empty() {
        let encoder = ByteStreamSplitEncoder::new();
        let data = Vec::new();
        let encoded = encoder.encode(&data).unwrap();
        assert!(encoded.is_empty());

        let decoded = encoder.decode(&encoded, 0).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_byte_stream_split_f32() {
        let encoder = ByteStreamSplitEncoder::new();

        // Create some f32 values
        let values = vec![1.0f32, 2.0f32, 3.0f32];
        let mut data = Vec::new();
        for &v in &values {
            data.extend_from_slice(&v.to_le_bytes());
        }

        let encoded = encoder.encode(&data).unwrap();
        let decoded = encoder.decode(&encoded, data.len()).unwrap();

        // Verify round trip
        assert_eq!(decoded, data);

        // Check that encoding actually rearranged bytes
        // For little-endian f32: [00 00 80 3f, 00 00 00 40, 00 00 40 40]
        // After split: stream0: [00,00,00], stream1: [00,00,40], stream2: [80,00,40], stream3: [3f,40,40]
        assert_ne!(encoded, data);
    }

    #[test]
    fn test_byte_stream_split_f64() {
        let encoder = ByteStreamSplitEncoder::new();

        // Create some f64 values
        let values = vec![1.0f64, 2.0f64];
        let mut data = Vec::new();
        for &v in &values {
            data.extend_from_slice(&v.to_le_bytes());
        }

        let encoded = encoder.encode(&data).unwrap();
        let decoded = encoder.decode(&encoded, data.len()).unwrap();

        assert_eq!(decoded, data);
        assert_ne!(encoded, data); // Should be rearranged
    }

    #[test]
    fn test_byte_stream_split_invalid_length() {
        let encoder = ByteStreamSplitEncoder::new();
        let data = vec![1, 2, 3]; // Not multiple of 4 or 8
        assert!(encoder.encode(&data).is_err());
    }
}