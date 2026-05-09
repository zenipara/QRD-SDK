//! Bit manipulation utilities

/// Bit manipulation operations
pub struct BitOps;

impl BitOps {
    /// Count trailing zeros in a u64
    pub fn trailing_zeros(value: u64) -> u32 {
        if value == 0 {
            64
        } else {
            value.trailing_zeros()
        }
    }

    /// Count leading zeros in a u64
    pub fn leading_zeros(value: u64) -> u32 {
        value.leading_zeros()
    }

    /// Find first set bit (bitscan forward)
    pub fn bitscan_forward(value: u64) -> Option<u32> {
        if value == 0 {
            None
        } else {
            Some(value.trailing_zeros())
        }
    }

    /// Find last set bit (bitscan reverse)
    pub fn bitscan_reverse(value: u64) -> Option<u32> {
        if value == 0 {
            None
        } else {
            Some(63 - value.leading_zeros())
        }
    }

    /// Count set bits (population count)
    pub fn popcount(value: u64) -> u32 {
        value.count_ones()
    }

    /// Reverse bits in a byte
    pub fn reverse_bits_byte(mut byte: u8) -> u8 {
        byte = (byte & 0xF0) >> 4 | (byte & 0x0F) << 4;
        byte = (byte & 0xCC) >> 2 | (byte & 0x33) << 2;
        byte = (byte & 0xAA) >> 1 | (byte & 0x55) << 1;
        byte
    }

    /// Pack bits from source array into destination
    /// Returns number of bytes written
    pub fn pack_bits(src: &[u8], dst: &mut [u8], bit_width: u8) -> usize {
        if bit_width == 0 || bit_width > 64 {
            return 0;
        }

        let mut dst_idx = 0;
        let mut dst_bit = 0;
        let mut dst_byte = 0u8;

        for &value in src {
            let value_bits = value as u64;

            let mut remaining_bits = bit_width;
            while remaining_bits > 0 {
                let bits_to_write = std::cmp::min(remaining_bits, 8 - dst_bit);
                let mask = (1u64 << bits_to_write) - 1;
                let bits = (value_bits >> (bit_width - remaining_bits)) & mask;

                dst_byte |= (bits as u8) << (8 - dst_bit - bits_to_write);
                dst_bit += bits_to_write;

                if dst_bit == 8 {
                    if dst_idx < dst.len() {
                        dst[dst_idx] = dst_byte;
                        dst_idx += 1;
                    }
                    dst_byte = 0;
                    dst_bit = 0;
                }

                remaining_bits -= bits_to_write;
            }
        }

        // Write remaining bits
        if dst_bit > 0 && dst_idx < dst.len() {
            dst[dst_idx] = dst_byte;
            dst_idx += 1;
        }

        dst_idx
    }

    /// Unpack bits from source array into destination
    /// Returns number of values unpacked
    pub fn unpack_bits(src: &[u8], dst: &mut [u8], bit_width: u8) -> usize {
        if bit_width == 0 || bit_width > 64 {
            return 0;
        }

        let mut src_idx = 0;
        let mut src_bit = 0;
        let mut dst_idx = 0;

        while dst_idx < dst.len() && src_idx < src.len() {
            let mut value = 0u64;
            let mut bits_read = 0;

            while bits_read < bit_width {
                if src_idx >= src.len() {
                    break;
                }

                let bits_available = 8 - src_bit;
                let bits_needed = bit_width - bits_read;
                let bits_to_read = std::cmp::min(bits_available, bits_needed);

                let mask = ((1u8 << bits_to_read) - 1) << (8 - src_bit - bits_to_read);
                let bits = (src[src_idx] & mask) >> (8 - src_bit - bits_to_read);

                value |= (bits as u64) << (bit_width - bits_read - bits_to_read);

                src_bit += bits_to_read;
                bits_read += bits_to_read;

                if src_bit == 8 {
                    src_idx += 1;
                    src_bit = 0;
                }
            }

            dst[dst_idx] = value as u8;
            dst_idx += 1;
        }

        dst_idx
    }

    /// Calculate minimum bits needed to represent a value
    pub fn min_bits_for_value(value: u64) -> u8 {
        if value == 0 {
            1
        } else {
            64 - value.leading_zeros()
        }
    }

    /// Calculate minimum bits needed to represent all values in array
    pub fn min_bits_for_values(values: &[u64]) -> u8 {
        let mut max_value = 0;
        for &value in values {
            if value > max_value {
                max_value = value;
            }
        }
        Self::min_bits_for_value(max_value)
    }

    /// Check if a number is a power of 2
    pub fn is_power_of_two(value: u64) -> bool {
        value != 0 && (value & (value - 1)) == 0
    }

    /// Round up to next power of 2
    pub fn next_power_of_two(value: u64) -> u64 {
        if value == 0 {
            return 1;
        }
        let mut v = value - 1;
        v |= v >> 1;
        v |= v >> 2;
        v |= v >> 4;
        v |= v >> 8;
        v |= v >> 16;
        v |= v >> 32;
        v + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailing_zeros() {
        assert_eq!(BitOps::trailing_zeros(0), 64);
        assert_eq!(BitOps::trailing_zeros(1), 0);
        assert_eq!(BitOps::trailing_zeros(2), 1);
        assert_eq!(BitOps::trailing_zeros(4), 2);
        assert_eq!(BitOps::trailing_zeros(8), 3);
    }

    #[test]
    fn test_leading_zeros() {
        assert_eq!(BitOps::leading_zeros(0), 64);
        assert_eq!(BitOps::leading_zeros(1), 63);
        assert_eq!(BitOps::leading_zeros(2), 62);
        assert_eq!(BitOps::leading_zeros(4), 61);
    }

    #[test]
    fn test_bitscan_forward() {
        assert_eq!(BitOps::bitscan_forward(0), None);
        assert_eq!(BitOps::bitscan_forward(1), Some(0));
        assert_eq!(BitOps::bitscan_forward(2), Some(1));
        assert_eq!(BitOps::bitscan_forward(6), Some(1)); // 110 in binary
    }

    #[test]
    fn test_bitscan_reverse() {
        assert_eq!(BitOps::bitscan_reverse(0), None);
        assert_eq!(BitOps::bitscan_reverse(1), Some(0));
        assert_eq!(BitOps::bitscan_reverse(2), Some(1));
        assert_eq!(BitOps::bitscan_reverse(6), Some(2)); // 110 in binary
    }

    #[test]
    fn test_popcount() {
        assert_eq!(BitOps::popcount(0), 0);
        assert_eq!(BitOps::popcount(1), 1);
        assert_eq!(BitOps::popcount(3), 2); // 11 in binary
        assert_eq!(BitOps::popcount(15), 4); // 1111 in binary
    }

    #[test]
    fn test_reverse_bits_byte() {
        assert_eq!(BitOps::reverse_bits_byte(0b00001111), 0b11110000);
        assert_eq!(BitOps::reverse_bits_byte(0b10101010), 0b01010101);
        assert_eq!(BitOps::reverse_bits_byte(0b11001100), 0b00110011);
    }

    #[test]
    fn test_pack_unpack_bits() {
        let original = vec![1u8, 2, 3, 4, 5];
        let mut packed = vec![0u8; 10];
        let mut unpacked = vec![0u8; 5];

        let packed_len = BitOps::pack_bits(&original, &mut packed, 3);
        assert!(packed_len > 0);

        let unpacked_len = BitOps::unpack_bits(&packed, &mut unpacked, 3);
        assert_eq!(unpacked_len, 5);

        for i in 0..5 {
            assert_eq!(unpacked[i], original[i]);
        }
    }

    #[test]
    fn test_min_bits_for_value() {
        assert_eq!(BitOps::min_bits_for_value(0), 1);
        assert_eq!(BitOps::min_bits_for_value(1), 1);
        assert_eq!(BitOps::min_bits_for_value(2), 2);
        assert_eq!(BitOps::min_bits_for_value(7), 3);
        assert_eq!(BitOps::min_bits_for_value(8), 4);
    }

    #[test]
    fn test_min_bits_for_values() {
        let values = vec![1, 5, 3, 7, 2];
        assert_eq!(BitOps::min_bits_for_values(&values), 3); // 7 needs 3 bits
    }

    #[test]
    fn test_is_power_of_two() {
        assert!(!BitOps::is_power_of_two(0));
        assert!(BitOps::is_power_of_two(1));
        assert!(BitOps::is_power_of_two(2));
        assert!(BitOps::is_power_of_two(4));
        assert!(BitOps::is_power_of_two(8));
        assert!(!BitOps::is_power_of_two(6));
        assert!(!BitOps::is_power_of_two(15));
    }

    #[test]
    fn test_next_power_of_two() {
        assert_eq!(BitOps::next_power_of_two(0), 1);
        assert_eq!(BitOps::next_power_of_two(1), 1);
        assert_eq!(BitOps::next_power_of_two(2), 2);
        assert_eq!(BitOps::next_power_of_two(3), 4);
        assert_eq!(BitOps::next_power_of_two(5), 8);
        assert_eq!(BitOps::next_power_of_two(7), 8);
        assert_eq!(BitOps::next_power_of_two(9), 16);
    }
}