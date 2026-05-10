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
