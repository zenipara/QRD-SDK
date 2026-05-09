//! DELTA_BINARY encoding implementation

use crate::encoding::Encoder;
use crate::error::{Error, Result};
use crate::utils::varint;

/// DELTA_BINARY encoder for sorted integer data
pub struct DeltaBinaryEncoder;

impl DeltaBinaryEncoder {
    /// Create new encoder
    pub fn new() -> Self {
        DeltaBinaryEncoder
    }
}

impl Encoder for DeltaBinaryEncoder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() % 8 != 0 {
            return Err(Error::EncodingError(
                "DELTA_BINARY requires data length to be multiple of 8 bytes (i64)".to_string(),
            ));
        }

        let values = decode_i64_slice(data)?;
        if values.is_empty() {
            return Ok(Vec::new());
        }

        // Check if sorted
        if !is_sorted(&values) {
            return Err(Error::EncodingError(
                "DELTA_BINARY requires sorted data".to_string(),
            ));
        }

        encode_delta_binary(&values)
    }

    fn decode(&self, data: &[u8], expected_length: usize) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let values = decode_delta_binary(data, expected_length / 8)?;
        encode_i64_slice(&values)
    }
}

/// Check if values are sorted in ascending order
fn is_sorted(values: &[i64]) -> bool {
    for i in 1..values.len() {
        if values[i] < values[i - 1] {
            return false;
        }
    }
    true
}

/// Encode values using delta-of-deltas
fn encode_delta_binary(values: &[i64]) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    if values.is_empty() {
        return Ok(result);
    }

    // Store minimum value
    result.extend_from_slice(&values[0].to_le_bytes());

    if values.len() == 1 {
        return Ok(result);
    }

    // Calculate deltas
    let mut deltas = Vec::with_capacity(values.len() - 1);
    for i in 1..values.len() {
        deltas.push(values[i] - values[i - 1]);
    }

    // Store first delta
    varint::encode_varint(&mut result, deltas[0] as i64);

    if deltas.len() == 1 {
        return Ok(result);
    }

    // Calculate delta-of-deltas
    let mut last_delta = deltas[0];
    for &delta in &deltas[1..] {
        let delta_of_delta = delta - last_delta;
        varint::encode_varint(&mut result, delta_of_delta);
        last_delta = delta;
    }

    Ok(result)
}

/// Decode delta-of-deltas encoded data
fn decode_delta_binary(data: &[u8], count: usize) -> Result<Vec<i64>> {
    if data.len() < 8 {
        return Err(Error::DecodingError(
            "DELTA_BINARY data too short for minimum value".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(count);
    let mut offset = 0;

    // Read minimum value
    let min_value = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
    result.push(min_value);
    offset += 8;

    if count == 1 {
        return Ok(result);
    }

    // Read first delta
    let (first_delta, delta_size) = varint::decode_varint(&data[offset..])?;
    result.push(min_value + first_delta);
    offset += delta_size;

    if count == 2 {
        return Ok(result);
    }

    let mut last_delta = first_delta;

    for _ in 2..count {
        let (delta_of_delta, size) = varint::decode_varint(&data[offset..])?;
        offset += size;

        let new_delta = last_delta + delta_of_delta;
        let new_value = result.last().unwrap() + new_delta;
        result.push(new_value);
        last_delta = new_delta;
    }

    Ok(result)
}

/// Decode slice of bytes as i64 values
fn decode_i64_slice(data: &[u8]) -> Result<Vec<i64>> {
    if data.len() % 8 != 0 {
        return Err(Error::EncodingError(
            "Data length must be multiple of 8 for i64 decoding".to_string(),
        ));
    }

    let mut values = Vec::with_capacity(data.len() / 8);
    for chunk in data.chunks_exact(8) {
        let value = i64::from_le_bytes(chunk.try_into().unwrap());
        values.push(value);
    }
    Ok(values)
}

/// Encode slice of i64 values as bytes
fn encode_i64_slice(values: &[i64]) -> Result<Vec<u8>> {
    let mut result = Vec::with_capacity(values.len() * 8);
    for &value in values {
        result.extend_from_slice(&value.to_le_bytes());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_binary_empty() {
        let encoder = DeltaBinaryEncoder::new();
        let data = Vec::new();
        let encoded = encoder.encode(&data).unwrap();
        assert!(encoded.is_empty());

        let decoded = encoder.decode(&encoded, 0).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_delta_binary_single_value() {
        let encoder = DeltaBinaryEncoder::new();
        let original = vec![42i64];
        let data = encode_i64_slice(&original).unwrap();

        let encoded = encoder.encode(&data).unwrap();
        assert_eq!(encoded.len(), 8); // Just the min value

        let decoded = encoder.decode(&encoded, 8).unwrap();
        let result = decode_i64_slice(&decoded).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_delta_binary_sorted_sequence() {
        let encoder = DeltaBinaryEncoder::new();
        let original = vec![100i64, 102, 105, 107, 110];
        let data = encode_i64_slice(&original).unwrap();

        let encoded = encoder.encode(&data).unwrap();
        let decoded = encoder.decode(&encoded, data.len()).unwrap();
        let result = decode_i64_slice(&decoded).unwrap();

        assert_eq!(result, original);
    }

    #[test]
    fn test_delta_binary_unsorted_fails() {
        let encoder = DeltaBinaryEncoder::new();
        let original = vec![100i64, 102, 101]; // Not sorted
        let data = encode_i64_slice(&original).unwrap();

        assert!(encoder.encode(&data).is_err());
    }

    #[test]
    fn test_is_sorted() {
        assert!(is_sorted(&[]));
        assert!(is_sorted(&[1]));
        assert!(is_sorted(&[1, 2, 3]));
        assert!(is_sorted(&[1, 1, 1]));
        assert!(!is_sorted(&[1, 3, 2]));
    }
}