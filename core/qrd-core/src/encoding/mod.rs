//! Encoding algorithms for columnar data

use crate::error::{Error, Result};
use std::fmt;

pub mod plain;
pub mod rle;
pub mod delta_binary;
pub mod bit_packed;
pub mod delta_byte_array;
pub mod dictionary_rle;
pub mod byte_stream_split;

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

/// Get encoder instance for encoding type
pub fn get_encoder(encoding: EncodingType) -> Result<Box<dyn Encoder>> {
    match encoding {
        EncodingType::Plain => Ok(Box::new(plain::PlainEncoder::new())),
        EncodingType::Rle => Ok(Box::new(rle::RleEncoder::new())),
        EncodingType::DeltaBinary => Ok(Box::new(delta_binary::DeltaBinaryEncoder::new())),
        EncodingType::BitPacked => Ok(Box::new(bit_packed::BitPackedEncoder::new())),
        EncodingType::DeltaByteArray => Ok(Box::new(delta_byte_array::DeltaByteArrayEncoder::new())),
        EncodingType::DictionaryRle => Ok(Box::new(dictionary_rle::DictionaryRleEncoder::new())),
        EncodingType::ByteStreamSplit => Ok(Box::new(byte_stream_split::ByteStreamSplitEncoder::new())),
        _ => Err(Error::EncodingError(format!("Encoder not implemented: {:?}", encoding))),
    }
}

/// Select appropriate encoding for a field type
pub fn select_encoding(field_type: &crate::schema::FieldType, data: &[u8]) -> EncodingType {
    match field_type {
        crate::schema::FieldType::Boolean => EncodingType::BitPacked,
        crate::schema::FieldType::String => {
            if is_sorted_strings(data) {
                EncodingType::DeltaByteArray
            } else if is_low_cardinality(data, 10) {
                EncodingType::DictionaryRle
            } else {
                EncodingType::Plain
            }
        }
        crate::schema::FieldType::Float32 | crate::schema::FieldType::Float64 => EncodingType::ByteStreamSplit,
        crate::schema::FieldType::Blob => EncodingType::Plain, // TODO: DELTA_BYTE_ARRAY for sorted blobs
        crate::schema::FieldType::Decimal => EncodingType::Plain, // TODO: Special decimal encoding
        // For integer types, check if sorted
        _ if is_integer_type(field_type) => {
            if is_sorted_integers(field_type, data) {
                EncodingType::DeltaBinary
            } else if is_low_cardinality_integers(field_type, data, 10) {
                EncodingType::DictionaryRle
            } else {
                EncodingType::Plain
            }
        }
        _ => EncodingType::Plain,
    }
}

/// Check if field type is an integer type
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

/// Check if integer data is sorted
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
fn is_low_cardinality_integers(field_type: &crate::schema::FieldType, data: &[u8], max_distinct: usize) -> bool {
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
}
