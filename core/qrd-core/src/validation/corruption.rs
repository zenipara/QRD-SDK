//! Corruption detection and recovery strategies

use crate::error::Result;

/// Types of potential corruption
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorruptionType {
    /// CRC32 mismatch on chunk
    ChecksumMismatch,
    /// Invalid offsets (out of bounds)
    InvalidOffset,
    /// Truncated file
    TruncatedFile,
    /// Invalid header/magic
    BadHeader,
    /// Row count mismatch
    RowCountMismatch,
    /// Column count mismatch
    ColumnCountMismatch,
    /// Invalid encoding type
    InvalidEncoding,
    /// Invalid compression codec
    InvalidCodec,
}

/// Corruption detection result
#[derive(Debug, Clone)]
pub struct CorruptionReport {
    /// Type of corruption detected
    pub corruption_type: CorruptionType,
    /// Location (byte offset if known)
    pub location: Option<u64>,
    /// Severity: true = fatal, false = recoverable
    pub fatal: bool,
    /// Description
    pub message: String,
}

impl CorruptionReport {
    /// Create new corruption report
    pub fn new(corruption_type: CorruptionType, message: impl Into<String>) -> Self {
        CorruptionReport {
            corruption_type,
            location: None,
            fatal: corruption_type != CorruptionType::ChecksumMismatch, // Most are fatal
            message: message.into(),
        }
    }

    /// Set location
    pub fn with_location(mut self, location: u64) -> Self {
        self.location = Some(location);
        self
    }

    /// Set fatality
    pub fn with_fatal(mut self, fatal: bool) -> Self {
        self.fatal = fatal;
        self
    }
}

/// Corruption detector
pub struct CorruptionDetector;

impl CorruptionDetector {
    /// Detect truncated file
    pub fn detect_truncation(file_size: u64, expected_min: u64) -> Option<CorruptionReport> {
        if file_size < expected_min {
            Some(
                CorruptionReport::new(
                    CorruptionType::TruncatedFile,
                    format!(
                        "File truncated: {} bytes but expected at least {}",
                        file_size, expected_min
                    ),
                )
                .with_fatal(true),
            )
        } else {
            None
        }
    }

    /// Detect invalid offsets
    pub fn detect_invalid_offset(offset: u64, file_size: u64) -> Option<CorruptionReport> {
        if offset >= file_size {
            Some(
                CorruptionReport::new(
                    CorruptionType::InvalidOffset,
                    format!("Offset {} exceeds file size {}", offset, file_size),
                )
                .with_location(offset)
                .with_fatal(true),
            )
        } else {
            None
        }
    }

    /// Detect row count mismatch
    pub fn detect_row_count_mismatch(
        header_count: u32,
        calculated_count: u32,
    ) -> Option<CorruptionReport> {
        if header_count != calculated_count {
            Some(
                CorruptionReport::new(
                    CorruptionType::RowCountMismatch,
                    format!(
                        "Row count mismatch: header={}, calculated={}",
                        header_count, calculated_count
                    ),
                )
                .with_fatal(false), // Recoverable: use calculated
            )
        } else {
            None
        }
    }

    /// Validate offsets are monotonically increasing
    pub fn validate_monotonic_offsets(offsets: &[u64]) -> Result<()> {
        let mut prev = 0u64;
        for (idx, &offset) in offsets.iter().enumerate() {
            if offset <= prev {
                return Err(crate::error::Error::InvalidData(format!(
                    "Row group offset {} out of order: {} <= {}",
                    idx, offset, prev
                )));
            }
            prev = offset;
        }
        Ok(())
    }

    /// Validate row group boundaries
    pub fn validate_row_group_bounds(rg_offsets: &[u64], file_size: u64) -> Result<()> {
        for (idx, &offset) in rg_offsets.iter().enumerate() {
            if offset >= file_size {
                return Err(crate::error::Error::InvalidData(format!(
                    "Row group {} starts at offset {} but file is only {} bytes",
                    idx, offset, file_size
                )));
            }
        }
        Ok(())
    }
}

/// Recovery strategies for corruption scenarios
pub struct RecoveryStrategy;

impl RecoveryStrategy {
    /// Recommend recovery action
    pub fn recommend(report: &CorruptionReport) -> RecoveryAction {
        if report.fatal {
            RecoveryAction::Abort
        } else {
            match report.corruption_type {
                CorruptionType::ChecksumMismatch => RecoveryAction::SkipChunk,
                CorruptionType::RowCountMismatch => RecoveryAction::UseCalculated,
                _ => RecoveryAction::Abort,
            }
        }
    }
}

/// Possible recovery actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Abort reading, return error
    Abort,
    /// Skip corrupted chunk, continue
    SkipChunk,
    /// Use recalculated value
    UseCalculated,
    /// Best effort - ignore error
    BestEffort,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_truncation() {
        let report = CorruptionDetector::detect_truncation(100, 200);
        assert!(report.is_some());
        assert!(report.unwrap().fatal);

        let report = CorruptionDetector::detect_truncation(200, 100);
        assert!(report.is_none());
    }

    #[test]
    fn test_detect_invalid_offset() {
        let report = CorruptionDetector::detect_invalid_offset(500, 100);
        assert!(report.is_some());
        assert!(report.unwrap().fatal);

        let report = CorruptionDetector::detect_invalid_offset(50, 100);
        assert!(report.is_none());
    }

    #[test]
    fn test_detect_row_count_mismatch() {
        let report = CorruptionDetector::detect_row_count_mismatch(1000, 1100);
        assert!(report.is_some());
        assert!(!report.unwrap().fatal);
    }

    #[test]
    fn test_monotonic_offsets() {
        let valid = vec![100, 500, 1000, 2000];
        assert!(CorruptionDetector::validate_monotonic_offsets(&valid).is_ok());

        let invalid = vec![100, 500, 400, 2000];
        assert!(CorruptionDetector::validate_monotonic_offsets(&invalid).is_err());
    }

    #[test]
    fn test_recovery_recommendation() {
        let fatal_report = CorruptionReport::new(CorruptionType::BadHeader, "test");
        assert_eq!(
            RecoveryStrategy::recommend(&fatal_report),
            RecoveryAction::Abort
        );

        let recoverable_report =
            CorruptionReport::new(CorruptionType::ChecksumMismatch, "test").with_fatal(false);
        assert_eq!(
            RecoveryStrategy::recommend(&recoverable_report),
            RecoveryAction::SkipChunk
        );
    }
}
