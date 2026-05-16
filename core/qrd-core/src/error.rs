//! Error types and Result type for QRD operations

use std::fmt;
use std::io;

/// Errors that can occur during QRD operations
#[derive(Debug)]
pub enum Error {
    /// I/O error
    Io(io::Error),
    /// Invalid magic bytes
    InvalidMagic,
    /// Unsupported version
    UnsupportedVersion {
        /// Major version encountered
        major: u16,
        /// Minor version encountered
        minor: u16,
    },
    /// Invalid schema
    InvalidSchema(String),
    /// Schema evolution error
    SchemaError(String),
    /// Invalid data
    InvalidData(String),
    /// Invalid input
    InvalidInput(String),
    /// Encoding error
    EncodingError(String),
    /// Decoding error
    DecodingError(String),
    /// Compression error
    CompressionError(String),
    /// Decompression error
    DecompressionError(String),
    /// Encryption error
    EncryptionError(String),
    /// Decryption error
    DecryptionError(String),
    /// ECC error
    EccError(String),
    /// Validation failed
    ValidationFailed(String),
    /// CRC mismatch
    CrcMismatch {
        /// Expected CRC32 value
        expected: u32,
        /// Actual CRC32 value
        actual: u32,
    },
    /// Row count mismatch
    RowCountMismatch {
        /// Expected number of rows
        expected: u32,
        /// Actual number of rows
        actual: u32,
    },
    /// Column count mismatch
    ColumnCountMismatch {
        /// Expected number of columns
        expected: u32,
        /// Actual number of columns
        actual: u32,
    },
    /// Type mismatch
    TypeMismatch(String),
    /// Configuration error
    ConfigError(String),
    /// Not found
    NotFound(String),
    /// Already closed
    AlreadyClosed,
    /// Other errors
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::InvalidMagic => write!(f, "Invalid QRD magic bytes"),
            Error::UnsupportedVersion { major, minor } => {
                write!(f, "Unsupported QRD version {}.{}", major, minor)
            }
            Error::InvalidSchema(msg) => write!(f, "Invalid schema: {}", msg),
            Error::SchemaError(msg) => write!(f, "Schema error: {}", msg),
            Error::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            Error::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Error::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            Error::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
            Error::CompressionError(msg) => write!(f, "Compression error: {}", msg),
            Error::DecompressionError(msg) => write!(f, "Decompression error: {}", msg),
            Error::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            Error::DecryptionError(msg) => write!(f, "Decryption error: {}", msg),
            Error::EccError(msg) => write!(f, "ECC error: {}", msg),
            Error::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            Error::CrcMismatch { expected, actual } => {
                write!(f, "CRC mismatch: expected {}, got {}", expected, actual)
            }
            Error::RowCountMismatch { expected, actual } => {
                write!(
                    f,
                    "Row count mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            Error::ColumnCountMismatch { expected, actual } => {
                write!(
                    f,
                    "Column count mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            Error::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            Error::ConfigError(msg) => write!(f, "Config error: {}", msg),
            Error::NotFound(msg) => write!(f, "Not found: {}", msg),
            Error::AlreadyClosed => write!(f, "Already closed"),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

/// Result type for QRD operations
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_io_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let qrd_error: Error = io_error.into();

        assert!(matches!(qrd_error, Error::Io(_)));
    }

    #[test]
    fn test_error_display_invalid_magic() {
        let error = Error::InvalidMagic;
        let message = format!("{}", error);
        assert_eq!(message, "Invalid QRD magic bytes");
    }

    #[test]
    fn test_error_display_unsupported_version() {
        let error = Error::UnsupportedVersion {
            major: 99,
            minor: 1,
        };
        let message = format!("{}", error);
        assert!(message.contains("99.1"));
    }

    #[test]
    fn test_error_display_invalid_schema() {
        let error = Error::InvalidSchema("test message".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Invalid schema"));
        assert!(message.contains("test message"));
    }

    #[test]
    fn test_error_display_invalid_data() {
        let error = Error::InvalidData("corrupted data".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Invalid data"));
        assert!(message.contains("corrupted data"));
    }

    #[test]
    fn test_error_display_invalid_input() {
        let error = Error::InvalidInput("bad input".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Invalid input"));
        assert!(message.contains("bad input"));
    }

    #[test]
    fn test_error_display_encoding_error() {
        let error = Error::EncodingError("encoding failed".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Encoding error"));
        assert!(message.contains("encoding failed"));
    }

    #[test]
    fn test_error_display_decoding_error() {
        let error = Error::DecodingError("decoding failed".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Decoding error"));
        assert!(message.contains("decoding failed"));
    }

    #[test]
    fn test_error_display_compression_error() {
        let error = Error::CompressionError("compression failed".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Compression error"));
    }

    #[test]
    fn test_error_display_decompression_error() {
        let error = Error::DecompressionError("decompression failed".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Decompression error"));
    }

    #[test]
    fn test_error_display_encryption_error() {
        let error = Error::EncryptionError("encryption failed".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Encryption error"));
    }

    #[test]
    fn test_error_display_decryption_error() {
        let error = Error::DecryptionError("decryption failed".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Decryption error"));
    }

    #[test]
    fn test_error_display_ecc_error() {
        let error = Error::EccError("ECC check failed".to_string());
        let message = format!("{}", error);
        assert!(message.contains("ECC error"));
    }

    #[test]
    fn test_error_display_validation_failed() {
        let error = Error::ValidationFailed("validation failed".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Validation failed"));
    }

    #[test]
    fn test_error_display_crc_mismatch() {
        let error = Error::CrcMismatch {
            expected: 0x12345678,
            actual: 0x87654321,
        };
        let message = format!("{}", error);
        assert!(message.contains("CRC mismatch"));
        assert!(message.contains("0x12345678") || message.contains("305419896"));
    }

    #[test]
    fn test_error_display_row_count_mismatch() {
        let error = Error::RowCountMismatch {
            expected: 100,
            actual: 50,
        };
        let message = format!("{}", error);
        assert!(message.contains("Row count mismatch"));
        assert!(message.contains("100"));
        assert!(message.contains("50"));
    }

    #[test]
    fn test_error_display_column_count_mismatch() {
        let error = Error::ColumnCountMismatch {
            expected: 5,
            actual: 3,
        };
        let message = format!("{}", error);
        assert!(message.contains("Column count mismatch"));
        assert!(message.contains("5"));
        assert!(message.contains("3"));
    }

    #[test]
    fn test_error_display_type_mismatch() {
        let error = Error::TypeMismatch("int32 vs string".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Type mismatch"));
        assert!(message.contains("int32 vs string"));
    }

    #[test]
    fn test_error_display_config_error() {
        let error = Error::ConfigError("invalid config".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Config error"));
        assert!(message.contains("invalid config"));
    }

    #[test]
    fn test_error_display_not_found() {
        let error = Error::NotFound("column not found".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Not found"));
        assert!(message.contains("column not found"));
    }

    #[test]
    fn test_error_display_already_closed() {
        let error = Error::AlreadyClosed;
        let message = format!("{}", error);
        assert_eq!(message, "Already closed");
    }

    #[test]
    fn test_error_display_other() {
        let error = Error::Other("generic error".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Error"));
        assert!(message.contains("generic error"));
    }

    #[test]
    fn test_error_is_std_error() {
        let error: Box<dyn std::error::Error> = Box::new(Error::InvalidMagic);
        assert!(!format!("{}", error).is_empty());
    }

    #[test]
    fn test_error_debug_impl() {
        let error = Error::InvalidSchema("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_result_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_err() {
        let result: Result<i32> = Err(Error::InvalidMagic);
        assert!(result.is_err());
    }

    #[test]
    fn test_io_error_propagation() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let qrd_err: Error = io_err.into();
        let message = format!("{}", qrd_err);
        assert!(message.contains("I/O error"));
    }

    #[test]
    fn test_error_deterministic_display() {
        let error1 = Error::InvalidData("test".to_string());
        let error2 = Error::InvalidData("test".to_string());

        let msg1 = format!("{}", error1);
        let msg2 = format!("{}", error2);

        assert_eq!(msg1, msg2);
    }

    #[test]
    fn test_error_variant_distinctness() {
        // Verify that different error variants are distinct
        let e1 = Error::InvalidMagic;
        let e2 = Error::AlreadyClosed;

        let s1 = format!("{:?}", e1);
        let s2 = format!("{:?}", e2);

        assert_ne!(s1, s2);
    }

    #[test]
    fn test_crc_mismatch_values() {
        let error = Error::CrcMismatch {
            expected: 0xFFFFFFFF,
            actual: 0x00000000,
        };

        let message = format!("{}", error);
        assert!(message.contains("CRC mismatch"));
    }

    #[test]
    fn test_error_chain_compatibility() {
        let result: Result<()> = Err(Error::ValidationFailed("chain test".to_string()));

        match result {
            Err(Error::ValidationFailed(msg)) => {
                assert_eq!(msg, "chain test");
            }
            _ => panic!("Unexpected error variant"),
        }
    }

    // Additional enterprise-grade error tests

    #[test]
    fn test_error_conversion() {
        // Test conversion from various error types
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let qrd_err: Error = io_err.into();
        assert!(matches!(qrd_err, Error::Io(_)));
    }

    #[test]
    fn test_error_propagation() {
        // Test that errors propagate correctly through Result
        fn failing_function() -> Result<i32> {
            Err(Error::InvalidData("test error".to_string()))
        }

        let result = failing_function();
        assert!(result.is_err());

        if let Err(Error::InvalidData(msg)) = result {
            assert_eq!(msg, "test error");
        } else {
            panic!("Wrong error type");
        }
    }

    #[test]
    fn test_error_formatting_stability() {
        // Test that error messages are stable
        let error = Error::EncodingError("test message".to_string());
        let msg1 = format!("{}", error);
        let msg2 = format!("{}", error);
        assert_eq!(msg1, msg2);
    }

    #[test]
    fn test_nested_errors() {
        // Test handling of nested error conditions
        let result: Result<()> = Err(Error::CompressionError(
            "nested compression error".to_string(),
        ));
        assert!(result.is_err());

        let error = result.unwrap_err();
        let message = format!("{}", error);
        assert!(message.contains("Compression error"));
    }

    #[test]
    fn test_parser_failure_mapping() {
        // Test mapping of parser failures to appropriate error types
        let invalid_data = "invalid data";
        let error = Error::InvalidData(invalid_data.to_string());
        let message = format!("{}", error);
        assert!(message.contains("Invalid data"));
        assert!(message.contains(invalid_data));
    }
}
