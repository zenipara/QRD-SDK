//! DELTA_BYTE_ARRAY encoding implementation

use crate::encoding::Encoder;
use crate::error::{Error, Result};
use crate::utils::varint;

/// DELTA_BYTE_ARRAY encoder for sorted variable-length byte arrays
pub struct DeltaByteArrayEncoder;

impl DeltaByteArrayEncoder {
    /// Create new encoder
    pub fn new() -> Self {
        DeltaByteArrayEncoder
    }
}

impl Encoder for DeltaByteArrayEncoder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Parse length-prefixed values
        let values = parse_length_prefixed_values(data)?;
        if values.is_empty() {
            return Ok(Vec::new());
        }

        // Check if sorted
        if !is_sorted_byte_arrays(&values) {
            return Err(Error::EncodingError(
                "DELTA_BYTE_ARRAY requires sorted byte arrays".to_string(),
            ));
        }

        encode_delta_byte_array(&values)
    }

    fn decode(&self, data: &[u8], expected_length: usize) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let values = decode_delta_byte_array(data, expected_length)?;
        serialize_length_prefixed_values(&values)
    }
}

/// Parse length-prefixed byte arrays from data
fn parse_length_prefixed_values(data: &[u8]) -> Result<Vec<Vec<u8>>> {
    let mut values = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        if offset + 4 > data.len() {
            return Err(Error::DecodingError("Invalid length prefix".to_string()));
        }

        let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + len > data.len() {
            return Err(Error::DecodingError(
                "Value length exceeds data".to_string(),
            ));
        }

        values.push(data[offset..offset + len].to_vec());
        offset += len;
    }

    Ok(values)
}

/// Serialize values with length prefixes
fn serialize_length_prefixed_values(values: &[Vec<u8>]) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    for value in values {
        result.extend_from_slice(&(value.len() as u32).to_le_bytes());
        result.extend_from_slice(value);
    }
    Ok(result)
}

/// Check if byte arrays are sorted lexicographically
fn is_sorted_byte_arrays(values: &[Vec<u8>]) -> bool {
    for i in 1..values.len() {
        if values[i] < values[i - 1] {
            return false;
        }
    }
    true
}

/// Encode using delta byte array
fn encode_delta_byte_array(values: &[Vec<u8>]) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    if values.is_empty() {
        return Ok(result);
    }

    // Store first value with length prefix
    varint::encode_varint(&mut result, values[0].len() as i64);
    result.extend_from_slice(&values[0]);

    for i in 1..values.len() {
        let prev = &values[i - 1];
        let curr = &values[i];

        // Find shared prefix length
        let shared_len = prev
            .iter()
            .zip(curr.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // Calculate suffix
        let suffix = &curr[shared_len..];

        // Encode shared length and suffix length
        varint::encode_varint(&mut result, shared_len as i64);
        varint::encode_varint(&mut result, suffix.len() as i64);
        result.extend_from_slice(suffix);
    }

    Ok(result)
}

/// Decode delta byte array
fn decode_delta_byte_array(data: &[u8], count: usize) -> Result<Vec<Vec<u8>>> {
    let mut result = Vec::with_capacity(count);
    let mut offset = 0;

    if count == 0 {
        return Ok(result);
    }

    // Read first value
    let (first_len, size) = varint::decode_varint(&data[offset..])?;
    offset += size;

    if offset + first_len as usize > data.len() {
        return Err(Error::DecodingError(
            "First value length exceeds data".to_string(),
        ));
    }

    let first_value = data[offset..offset + first_len as usize].to_vec();
    result.push(first_value.clone());
    offset += first_len as usize;

    let mut prev = first_value;

    for _ in 1..count {
        // Read shared length
        let (shared_len, size) = varint::decode_varint(&data[offset..])?;
        offset += size;

        // Read suffix length
        let (suffix_len, size) = varint::decode_varint(&data[offset..])?;
        offset += size;

        if offset + suffix_len as usize > data.len() {
            return Err(Error::DecodingError(
                "Suffix length exceeds data".to_string(),
            ));
        }

        // Reconstruct value
        let mut current = prev[0..shared_len as usize].to_vec();
        current.extend_from_slice(&data[offset..offset + suffix_len as usize]);
        offset += suffix_len as usize;

        result.push(current.clone());
        prev = current;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_byte_array_empty() {
        let encoder = DeltaByteArrayEncoder::new();
        let data = Vec::new();
        let encoded = encoder.encode(&data).unwrap();
        assert!(encoded.is_empty());

        let decoded = encoder.decode(&encoded, 0).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_delta_byte_array_single_value() {
        let encoder = DeltaByteArrayEncoder::new();
        let original = vec![b"hello".to_vec()];
        let data = serialize_length_prefixed_values(&original).unwrap();

        let encoded = encoder.encode(&data).unwrap();
        let decoded = encoder.decode(&encoded, 1).unwrap();
        let result = parse_length_prefixed_values(&decoded).unwrap();

        assert_eq!(result, original);
    }

    #[test]
    fn test_delta_byte_array_sorted_strings() {
        let encoder = DeltaByteArrayEncoder::new();
        let original = vec![
            b"apple".to_vec(),
            b"apply".to_vec(),
            b"apricot".to_vec(),
            b"banana".to_vec(),
        ];
        let data = serialize_length_prefixed_values(&original).unwrap();

        let encoded = encoder.encode(&data).unwrap();
        let decoded = encoder.decode(&encoded, 4).unwrap();
        let result = parse_length_prefixed_values(&decoded).unwrap();

        assert_eq!(result, original);
    }

    #[test]
    fn test_delta_byte_array_unsorted_fails() {
        let encoder = DeltaByteArrayEncoder::new();
        let original = vec![
            b"apple".to_vec(),
            b"banana".to_vec(),
            b"apply".to_vec(), // Not sorted
        ];
        let data = serialize_length_prefixed_values(&original).unwrap();

        assert!(encoder.encode(&data).is_err());
    }

    #[test]
    fn test_is_sorted_byte_arrays() {
        assert!(is_sorted_byte_arrays(&[]));
        assert!(is_sorted_byte_arrays(&[b"a".to_vec()]));
        assert!(is_sorted_byte_arrays(&[
            b"a".to_vec(),
            b"b".to_vec(),
            b"c".to_vec()
        ]));
        assert!(!is_sorted_byte_arrays(&[
            b"c".to_vec(),
            b"b".to_vec(),
            b"a".to_vec()
        ]));
    }
}
