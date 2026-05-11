//! Utilities and helper functions

use crate::error::Result;
use crate::schema::Schema;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

pub mod bit_ops;
/// SIMD and scalar implementations for performance-critical operations
pub mod simd;

/// Write QRD file header to a writer
pub(crate) fn write_header(
    writer: &mut dyn Write,
    schema: &Schema,
    row_group_size: u32,
) -> Result<()> {
    // Magic bytes
    writer.write_all(crate::QRD_MAGIC)?;
    // Version
    writer.write_u16::<LittleEndian>(crate::QRD_VERSION_MAJOR)?;
    writer.write_u16::<LittleEndian>(crate::QRD_VERSION_MINOR)?;
    // Schema ID
    writer.write_u32::<LittleEndian>(schema.schema_id)?;
    // Created timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32;
    writer.write_u32::<LittleEndian>(now)?;
    // Placeholder for row count — sentinel indicates value stored in footer
    writer.write_u32::<LittleEndian>(u32::MAX)?;
    // Column count
    writer.write_u32::<LittleEndian>(schema.fields.len() as u32)?;
    // Row group size
    writer.write_u32::<LittleEndian>(row_group_size)?;
    // Reserved
    writer.write_u32::<LittleEndian>(0)?;

    Ok(())
}

/// Varint encoding/decoding utilities
pub mod varint {
    /// Encode u64 as varint
    pub fn encode(mut value: u64) -> Vec<u8> {
        let mut result = Vec::new();

        while value > 0x7F {
            result.push((value as u8) | 0x80);
            value >>= 7;
        }
        result.push(value as u8);

        result
    }

    /// Decode varint from bytes, returns (value, bytes_read)
    pub fn decode(data: &[u8]) -> Option<(u64, usize)> {
        let mut result = 0u64;
        let mut shift = 0;
        let mut i = 0;

        loop {
            if i >= data.len() {
                return None;
            }

            let byte = data[i];
            result |= ((byte & 0x7F) as u64) << shift;

            if byte & 0x80 == 0 {
                return Some((result, i + 1));
            }

            shift += 7;
            i += 1;

            if shift > 63 {
                return None; // Overflow
            }
        }
    }

    /// Encode i64 as zigzag varint
    pub fn encode_varint(buf: &mut Vec<u8>, value: i64) {
        let zigzag = ((value << 1) ^ (value >> 63)) as u64;
        buf.extend(encode(zigzag));
    }

    /// Decode zigzag varint, returns (value, bytes_read)
    pub fn decode_varint(data: &[u8]) -> Result<(i64, usize), crate::error::Error> {
        if let Some((encoded, bytes)) = decode(data) {
            // Zigzag decode
            let value = (encoded >> 1) as i64 ^ -((encoded & 1) as i64);
            Ok((value, bytes))
        } else {
            Err(crate::error::Error::DecodingError(
                "Invalid varint".to_string(),
            ))
        }
    }
}

/// Bit packing utilities
pub mod bits {
    /// Pack booleans into bytes (8 bits per byte)
    pub fn pack_booleans(bools: &[bool]) -> Vec<u8> {
        let byte_count = (bools.len() + 7) / 8;
        let mut result = vec![0u8; byte_count];

        for (i, &is_true) in bools.iter().enumerate() {
            if is_true {
                let byte_idx = i / 8;
                let bit_idx = i % 8;
                result[byte_idx] |= 1 << bit_idx;
            }
        }

        result
    }

    /// Unpack bytes into booleans
    pub fn unpack_booleans(bytes: &[u8], count: usize) -> Vec<bool> {
        let mut result = Vec::with_capacity(count);

        for i in 0..count {
            let byte_idx = i / 8;
            let bit_idx = i % 8;

            if byte_idx < bytes.len() {
                let bit = (bytes[byte_idx] >> bit_idx) & 1;
                result.push(bit != 0);
            }
        }

        result
    }
}

/// Read/write helpers
pub mod rwhelper {
    use crate::error::Result;
    use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
    use std::io::Cursor;

    /// Read u32 little-endian
    pub fn read_u32_le(data: &[u8]) -> Result<u32> {
        let mut cursor = Cursor::new(data);
        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    /// Write u32 little-endian
    pub fn write_u32_le(value: u32) -> Vec<u8> {
        let mut v = Vec::new();
        v.write_u32::<LittleEndian>(value).unwrap();
        v
    }

    /// Read u64 little-endian
    pub fn read_u64_le(data: &[u8]) -> Result<u64> {
        let mut cursor = Cursor::new(data);
        Ok(cursor.read_u64::<LittleEndian>()?)
    }

    /// Write u64 little-endian
    pub fn write_u64_le(value: u64) -> Vec<u8> {
        let mut v = Vec::new();
        v.write_u64::<LittleEndian>(value).unwrap();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_encode_decode() {
        for value in &[0u64, 1, 127, 128, 16384, 1000000] {
            let encoded = varint::encode(*value);
            let (decoded, _) = varint::decode(&encoded).unwrap();
            assert_eq!(decoded, *value);
        }
    }

    #[test]
    fn test_bit_packing() {
        let bools = vec![true, false, true, true, false, false, true, false];
        let packed = bits::pack_booleans(&bools);
        let unpacked = bits::unpack_booleans(&packed, bools.len());
        assert_eq!(unpacked, bools);
    }
}
