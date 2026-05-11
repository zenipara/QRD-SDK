//! QRD Core Engine
//!
//! Production-grade columnar binary storage format with streaming support,
//! deterministic encoding, comprehensive compression, and optional encryption.
//!
//! # Architecture
//!
//! ```text
//! Schema → Writer → Encoder → Compressor → File
//!            ↓
//!        Row Buffering
//!            ↓
//!        Row → Column Transposition
//!            ↓
//!        Row Group Flushing
//! ```
//!
//! # Quick Start
//!
//! ```no_run
//! use qrd_core::prelude::*;
//!
//! # fn main() -> Result<()> {
//! // Define schema
//! let schema = SchemaBuilder::new()
//!     .add_field("id", FieldType::Int64, Nullability::Required)?
//!     .add_field("name", FieldType::String, Nullability::Optional)?
//!     .build()?;
//!
//! // Create writer
//! let mut writer = FileWriter::new("output.qrd", schema)?;
//!
//! // Write rows - each field as raw bytes
//! writer.write_row(vec![
//!     1i64.to_le_bytes().to_vec(),
//!     b"Alice".to_vec()
//! ])?;
//! writer.write_row(vec![
//!     2i64.to_le_bytes().to_vec(),
//!     b"Bob".to_vec()
//! ])?;
//!
//! // Finalize
//! writer.finish()?;
//!
//! // Read back
//! let reader = FileReader::new("output.qrd")?;
//! let rows = reader.rows()?;
//! for row in rows {
//!     println!("{:?}", row);
//! }
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs, rust_2018_idioms, rustdoc::missing_crate_level_docs)]

pub mod columnar;
pub mod compression;
pub mod ecc;
pub mod encoding;
pub mod encryption;
pub mod error;
pub mod footer;
pub mod io;
pub mod memory_profiling;
pub mod metadata;
pub mod reader;
pub mod rowgroup;
pub mod schema;
pub mod test_vectors;
pub mod utils;
pub mod validation;
pub mod writer;

pub mod prelude {
    //! Convenient re-exports of common types
    pub use crate::compression::CompressionCodec;
    pub use crate::encoding::EncodingType;
    pub use crate::error::{Error, Result};
    pub use crate::reader::FileReader;
    pub use crate::schema::{FieldType, Nullability, Schema, SchemaBuilder};
    pub use crate::validation::Validator;
    pub use crate::writer::FileWriter;
}

pub use error::{Error, Result};

/// QRD Format Version
pub const QRD_VERSION_MAJOR: u16 = 1;
/// Minor version number for QRD format
pub const QRD_VERSION_MINOR: u16 = 0;

/// QRD Magic Bytes: "QRD\x01"
pub const QRD_MAGIC: &[u8; 4] = b"QRD\x01";

/// Default row group size (in rows)
pub const DEFAULT_ROW_GROUP_SIZE: u32 = 100_000;

/// Default buffer size for I/O operations
pub const DEFAULT_BUFFER_SIZE: usize = 8 * 1024 * 1024; // 8MB

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_constants_exist() {
        assert_eq!(QRD_MAGIC[0], b'Q');
        assert_eq!(QRD_MAGIC[1], b'R');
        assert_eq!(QRD_MAGIC[2], b'D');
        assert_eq!(QRD_MAGIC[3], 0x01);
    }
}
