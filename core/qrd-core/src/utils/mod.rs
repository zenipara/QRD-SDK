//! Utilities and helper functions

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
    use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
    use std::io::Cursor;
    use crate::error::Result;

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
