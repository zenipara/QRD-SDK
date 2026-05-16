//! Validation and corruption detection
//!
//! Provides:
//! - CRC32 and integrity checks
//! - Corruption detection
//! - Recovery strategies
//! - Malformed input handling
//! - Bloom filters and indexing for predicate pushdown

pub mod corruption;
pub mod index;

pub use corruption::{
    CorruptionDetector, CorruptionReport, CorruptionType, RecoveryAction, RecoveryStrategy,
};
pub use index::{
    BloomFilter, BloomFilterStats, CompositeIndex, HashIndex, IndexStats, Predicate,
    PredicatePushdownResult, RangeIndex,
};

use crate::error::Result;
use crc32fast::Hasher;

/// Validator for integrity checks
pub struct Validator;

impl Validator {
    /// Calculate CRC32 of data
    pub fn calculate_crc32(data: &[u8]) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize()
    }

    /// Verify CRC32
    pub fn verify_crc32(data: &[u8], expected_crc: u32) -> Result<()> {
        let actual_crc = Self::calculate_crc32(data);
        if actual_crc != expected_crc {
            return Err(crate::error::Error::CrcMismatch {
                expected: expected_crc,
                actual: actual_crc,
            });
        }
        Ok(())
    }

    /// Validate QRD magic bytes
    pub fn validate_magic(magic: &[u8; 4]) -> Result<()> {
        if magic != crate::QRD_MAGIC {
            return Err(crate::error::Error::InvalidMagic);
        }
        Ok(())
    }

    /// Validate version compatibility
    pub fn validate_version(major: u16, minor: u16) -> Result<()> {
        if major > crate::QRD_VERSION_MAJOR {
            return Err(crate::error::Error::UnsupportedVersion { major, minor });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_calculation() {
        let data = b"hello world";
        let crc1 = Validator::calculate_crc32(data);
        let crc2 = Validator::calculate_crc32(data);
        assert_eq!(crc1, crc2);
    }

    #[test]
    fn test_crc32_verification() {
        let data = b"hello world";
        let crc = Validator::calculate_crc32(data);
        assert!(Validator::verify_crc32(data, crc).is_ok());
        assert!(Validator::verify_crc32(data, crc + 1).is_err());
    }

    #[test]
    fn test_magic_validation() {
        assert!(Validator::validate_magic(crate::QRD_MAGIC).is_ok());
        let invalid_magic = b"XXXX";
        assert!(Validator::validate_magic(invalid_magic).is_err());
    }
}
