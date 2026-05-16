//! Encoding algorithms for columnar data

use crate::error::{Error, Result};
use std::fmt;

pub mod bit_packed;
pub mod byte_stream_split;
pub mod delta_binary;
pub mod delta_byte_array;
pub mod dictionary_rle;
pub mod plain;
pub mod rle;

/// PASSTHROUGH encoder for pre-serialized data.
pub struct PassthroughEncoder;

impl PassthroughEncoder {
    /// Create new passthrough encoder.
    pub fn new() -> Self {
        PassthroughEncoder
    }
}

impl Default for PassthroughEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder for PassthroughEncoder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    fn decode(&self, data: &[u8], _expected_length: usize) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }
}

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
            _ => Err(Error::EncodingError(format!("Unknown encoding ID: {}", id))),
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

/// Get encoder instance for encoding type
pub fn get_encoder(encoding: EncodingType) -> Result<Box<dyn Encoder>> {
    match encoding {
        EncodingType::Plain => Ok(Box::new(plain::PlainEncoder::new())),
        EncodingType::Rle => Ok(Box::new(rle::RleEncoder::new())),
        EncodingType::DeltaBinary => Ok(Box::new(delta_binary::DeltaBinaryEncoder::new())),
        EncodingType::BitPacked => Ok(Box::new(bit_packed::BitPackedEncoder::new())),
        EncodingType::DeltaByteArray => {
            Ok(Box::new(delta_byte_array::DeltaByteArrayEncoder::new()))
        }
        EncodingType::DictionaryRle => Ok(Box::new(dictionary_rle::DictionaryRleEncoder::new())),
        EncodingType::ByteStreamSplit => {
            Ok(Box::new(byte_stream_split::ByteStreamSplitEncoder::new()))
        }
        EncodingType::Passthrough => Ok(Box::new(PassthroughEncoder::new())),
        // All variants handled above; no fallback required.
    }
}

/// Select appropriate encoding for a field type
pub fn select_encoding(field_type: &crate::schema::FieldType, _data: &[u8]) -> EncodingType {
    match field_type {
        crate::schema::FieldType::Enum => EncodingType::DictionaryRle,
        crate::schema::FieldType::String => EncodingType::Plain,
        _ => match field_type.fixed_size() {
            Some(_) => EncodingType::Plain,
            None => EncodingType::Passthrough,
        },
    }
}

/// Check if field type is an integer type
#[allow(dead_code)]
fn is_integer_type(field_type: &crate::schema::FieldType) -> bool {
    matches!(
        field_type,
        crate::schema::FieldType::Int8
            | crate::schema::FieldType::Int16
            | crate::schema::FieldType::Int32
            | crate::schema::FieldType::Int64
            | crate::schema::FieldType::UInt8
            | crate::schema::FieldType::UInt16
            | crate::schema::FieldType::UInt32
            | crate::schema::FieldType::UInt64
            | crate::schema::FieldType::Timestamp
            | crate::schema::FieldType::Date
            | crate::schema::FieldType::Time
            | crate::schema::FieldType::Duration
    )
}

/// Read integer value at index
#[allow(dead_code)]
fn read_int_at(field_type: &crate::schema::FieldType, data: &[u8], index: usize) -> i64 {
    if let Some(size) = field_type.fixed_size() {
        let offset = index * size;
        if offset + size > data.len() {
            return 0;
        }
        match field_type {
            crate::schema::FieldType::Int8 => data[offset] as i8 as i64,
            crate::schema::FieldType::Int16 => {
                i16::from_le_bytes([data[offset], data[offset + 1]]) as i64
            }
            crate::schema::FieldType::Int32 => i32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as i64,
            crate::schema::FieldType::Int64 => i64::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]),
            _ => 0,
        }
    } else {
        0
    }
}

/// Check if string data is sorted
#[allow(dead_code)]
fn is_sorted_strings(data: &[u8]) -> bool {
    let mut offset = 0;
    let mut last_string: Option<Vec<u8>> = None;

    while offset < data.len() {
        if offset + 4 > data.len() {
            return false;
        }
        let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + len > data.len() {
            return false;
        }

        let current_string = data[offset..offset + len].to_vec();
        if let Some(ref last) = last_string {
            if current_string < *last {
                return false;
            }
        }
        last_string = Some(current_string);
        offset += len;
    }

    true
}

/// Check if integer data is sorted
#[allow(dead_code)]
fn is_sorted_integers(field_type: &crate::schema::FieldType, data: &[u8]) -> bool {
    if let Some(size) = field_type.fixed_size() {
        if data.len() % size != 0 {
            return false;
        }
        let count = data.len() / size;
        if count < 2 {
            return true;
        }

        for i in 1..count {
            let prev = read_int_at(field_type, data, i - 1);
            let curr = read_int_at(field_type, data, i);
            if curr < prev {
                return false;
            }
        }
        true
    } else {
        false
    }
}

/// Check if string data has low cardinality
#[allow(dead_code)]
fn is_low_cardinality(data: &[u8], max_distinct: usize) -> bool {
    let mut seen = std::collections::HashSet::new();
    let mut offset = 0;

    while offset < data.len() && seen.len() <= max_distinct {
        if offset + 4 > data.len() {
            return false;
        }
        let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + len > data.len() {
            return false;
        }

        seen.insert(&data[offset..offset + len]);
        offset += len;
    }

    seen.len() <= max_distinct
}

/// Check if integer data has low cardinality
#[allow(dead_code)]
fn is_low_cardinality_integers(
    field_type: &crate::schema::FieldType,
    data: &[u8],
    max_distinct: usize,
) -> bool {
    if let Some(size) = field_type.fixed_size() {
        if data.len() % size != 0 {
            return false;
        }
        let mut seen = std::collections::HashSet::new();
        for chunk in data.chunks(size) {
            seen.insert(chunk);
            if seen.len() > max_distinct {
                return false;
            }
        }
        true
    } else {
        false
    }
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

    #[test]
    fn test_invalid_encoding_id_rejection() {
        let result = EncodingType::from_id(255);
        assert!(result.is_err());

        let result2 = EncodingType::from_id(99);
        assert!(result2.is_err());
    }

    #[test]
    fn test_encoding_type_display() {
        assert_eq!(format!("{}", EncodingType::Plain), "PLAIN");
        assert_eq!(format!("{}", EncodingType::Rle), "RLE");
        assert_eq!(format!("{}", EncodingType::BitPacked), "BIT_PACKED");
        assert_eq!(format!("{}", EncodingType::DeltaBinary), "DELTA_BINARY");
        assert_eq!(
            format!("{}", EncodingType::DeltaByteArray),
            "DELTA_BYTE_ARRAY"
        );
        assert_eq!(
            format!("{}", EncodingType::ByteStreamSplit),
            "BYTE_STREAM_SPLIT"
        );
        assert_eq!(format!("{}", EncodingType::DictionaryRle), "DICTIONARY_RLE");
        assert_eq!(format!("{}", EncodingType::Passthrough), "PASSTHROUGH");
    }

    #[test]
    fn test_passthrough_encoder_identity() {
        let encoder = PassthroughEncoder::new();
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let encoded = encoder.encode(&data).unwrap();
        assert_eq!(encoded, data);

        let decoded = encoder.decode(&encoded, data.len()).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_passthrough_encoder_empty_data() {
        let encoder = PassthroughEncoder::new();
        let data = vec![];

        let encoded = encoder.encode(&data).unwrap();
        assert_eq!(encoded.len(), 0);

        let decoded = encoder.decode(&encoded, 0).unwrap();
        assert_eq!(decoded.len(), 0);
    }

    #[test]
    fn test_passthrough_encoder_large_data() {
        let encoder = PassthroughEncoder::new();
        let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();

        let encoded = encoder.encode(&data).unwrap();
        assert_eq!(encoded.len(), data.len());
        assert_eq!(encoded, data);
    }

    #[test]
    fn test_encoding_type_all_valid_ids() {
        let valid_ids = vec![0, 1, 2, 3, 4, 5, 6, 7];

        for id in valid_ids {
            let encoding = EncodingType::from_id(id);
            assert!(encoding.is_ok());

            let enc = encoding.unwrap();
            assert_eq!(enc.to_id(), id);
        }
    }

    #[test]
    fn test_encoding_type_deterministic_to_id() {
        let encoding = EncodingType::DeltaBinary;
        let id1 = encoding.to_id();
        let id2 = encoding.to_id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_low_cardinality_detection_true() {
        let data = vec![
            b"apple".to_vec(),
            b"apple".to_vec(),
            b"banana".to_vec(),
            b"banana".to_vec(),
        ];

        let mut encoded = Vec::new();
        for item in data {
            encoded.extend_from_slice(&(item.len() as u32).to_le_bytes());
            encoded.extend_from_slice(&item);
        }

        assert!(is_low_cardinality(&encoded, 5));
    }

    #[test]
    fn test_low_cardinality_detection_false() {
        let data: Vec<Vec<u8>> = (0..100)
            .map(|i| format!("value_{}", i).into_bytes())
            .collect();

        let mut encoded = Vec::new();
        for item in data {
            encoded.extend_from_slice(&(item.len() as u32).to_le_bytes());
            encoded.extend_from_slice(&item);
        }

        assert!(!is_low_cardinality(&encoded, 5));
    }

    #[test]
    fn test_low_cardinality_integers_true() {
        let values = vec![1i64, 1i64, 2i64, 2i64, 1i64];
        let mut data = Vec::new();
        for v in values {
            data.extend_from_slice(&v.to_le_bytes());
        }

        assert!(is_low_cardinality_integers(
            &crate::schema::FieldType::Int64,
            &data,
            5
        ));
    }

    #[test]
    fn test_low_cardinality_integers_false() {
        let values: Vec<i64> = (0..100).collect();
        let mut data = Vec::new();
        for v in values {
            data.extend_from_slice(&v.to_le_bytes());
        }

        assert!(!is_low_cardinality_integers(
            &crate::schema::FieldType::Int64,
            &data,
            5
        ));
    }

    #[test]
    fn test_encoding_type_equality() {
        assert_eq!(EncodingType::Plain, EncodingType::Plain);
        assert_ne!(EncodingType::Plain, EncodingType::Rle);
        assert_ne!(EncodingType::DeltaBinary, EncodingType::DeltaByteArray);
    }

    #[test]
    fn test_encoding_type_clone() {
        let enc1 = EncodingType::DictionaryRle;
        let enc2 = enc1.clone();
        assert_eq!(enc1, enc2);
    }

    #[test]
    fn test_encoding_type_copy() {
        let enc1 = EncodingType::ByteStreamSplit;
        let enc2 = enc1;
        assert_eq!(enc1, enc2);
    }

    #[test]
    fn test_passthrough_encoder_deterministic() {
        let encoder = PassthroughEncoder::new();
        let data = b"deterministic_test".to_vec();

        let encoded1 = encoder.encode(&data).unwrap();
        let encoded2 = encoder.encode(&data).unwrap();

        assert_eq!(encoded1, encoded2);
    }

    #[test]
    fn test_low_cardinality_empty_input() {
        assert!(is_low_cardinality(&[], 10));
    }

    #[test]
    fn test_low_cardinality_integers_empty_input() {
        let empty: Vec<u8> = vec![];
        assert!(is_low_cardinality_integers(
            &crate::schema::FieldType::Int64,
            &empty,
            10
        ));
    }

    #[test]
    fn test_low_cardinality_malformed_length_prefix() {
        let malformed = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF]; // Invalid length prefix
        assert!(!is_low_cardinality(&malformed, 5));
    }

    #[test]
    fn test_low_cardinality_integers_invalid_size() {
        let data = vec![1, 2, 3]; // Not divisible by 8 (i64 size)
        assert!(!is_low_cardinality_integers(
            &crate::schema::FieldType::Int64,
            &data,
            5
        ));
    }

    // Additional enterprise-grade tests

    #[test]
    fn test_encoding_auto_selection_fallback() {
        let data = vec![0u8; 1000]; // High cardinality but compressible
        let encoding = select_encoding(&crate::schema::FieldType::Int64, &data);
        // Should not crash, should select some valid encoding
        assert!(matches!(
            encoding,
            EncodingType::Plain
                | EncodingType::Rle
                | EncodingType::BitPacked
                | EncodingType::DeltaBinary
                | EncodingType::DictionaryRle
        ));
    }

    #[test]
    fn test_invalid_encoding_ids_rejection() {
        for invalid_id in [255, 100, 50, 20] {
            assert!(EncodingType::from_id(invalid_id).is_err());
        }
    }

    #[test]
    fn test_encoding_fallback_behavior() {
        // Test that invalid encoding types don't cause panics
        let invalid_id = 99;
        let result = EncodingType::from_id(invalid_id);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::Error::EncodingError(_)
        ));
    }

    #[test]
    fn test_encoding_deterministic_selection() {
        let data = vec![
            1i64.to_le_bytes().to_vec(),
            2i64.to_le_bytes().to_vec(),
            1i64.to_le_bytes().to_vec(),
        ];
        let mut encoded = Vec::new();
        for item in &data {
            encoded.extend_from_slice(&(item.len() as u32).to_le_bytes());
            encoded.extend_from_slice(item);
        }

        let enc1 = select_encoding(&crate::schema::FieldType::Int64, &encoded);
        let enc2 = select_encoding(&crate::schema::FieldType::Int64, &encoded);
        assert_eq!(enc1, enc2);
    }

    #[test]
    fn test_encoding_roundtrip_integrity() {
        // Test that encoding roundtrip preserves data
        let data = b"test data for roundtrip";
        let encoder = PassthroughEncoder::new();

        let encoded = encoder.encode(data).unwrap();
        let decoded = encoder.decode(&encoded, data.len()).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_mixed_type_handling() {
        // Test encoding selection with mixed types (though QRD is columnar)
        let string_data = b"string data";
        let enc = select_encoding(&crate::schema::FieldType::String, string_data);
        assert!(matches!(
            enc,
            EncodingType::Plain | EncodingType::DictionaryRle
        ));
    }

    #[test]
    fn test_edge_case_cardinality() {
        // Test with exactly threshold cardinality
        let mut data = Vec::new();
        for i in 0..10 {
            // Exactly threshold
            let val = format!("value_{}", i).into_bytes();
            data.extend_from_slice(&(val.len() as u32).to_le_bytes());
            data.extend_from_slice(&val);
        }

        let enc = select_encoding(&crate::schema::FieldType::String, &data);
        // Should handle boundary case without panic
        assert!(matches!(
            enc,
            EncodingType::Plain | EncodingType::DictionaryRle
        ));
    }

    #[test]
    fn test_empty_inputs_encoding() {
        let empty: Vec<u8> = vec![];
        let enc = select_encoding(&crate::schema::FieldType::Int64, &empty);
        // Should not crash on empty input
        assert!(matches!(
            enc,
            EncodingType::Plain
                | EncodingType::Rle
                | EncodingType::BitPacked
                | EncodingType::DeltaBinary
                | EncodingType::DictionaryRle
        ));
    }

    #[test]
    fn test_malformed_encoded_payloads() {
        // Test that malformed payloads don't cause undefined behavior
        let malformed = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid data
        let encoder = PassthroughEncoder::new();

        // Should handle gracefully
        let result = encoder.decode(&malformed, 4);
        assert!(result.is_ok()); // Passthrough just returns the data
    }
}
