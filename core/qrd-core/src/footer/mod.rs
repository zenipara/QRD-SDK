//! Footer metadata and serialization
//!
//! Provides:
//! - FooterBuilder: Deterministic footer construction
//! - FooterParser: Random access and footer parsing
//! - Footer: Core serialization

pub mod builder;
pub mod parser;

pub use builder::FooterBuilder;
pub use parser::FooterParser;

use crate::schema::Schema;
use crate::error::Result;
use crate::metadata::MetadataIndex;
use serde::{Deserialize, Serialize};

/// Footer structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footer {
    /// Schema definition
    pub schema: Schema,
    /// Row group offsets in file
    pub row_group_offsets: Vec<u64>,
    /// Total rows in file
    pub row_count: u32,
    /// Creation timestamp (Unix seconds)
    pub created_at: u32,
    /// Modification timestamp (Unix seconds)
    pub modified_at: u32,
    /// Metadata index for efficient access
    pub metadata_index: Option<MetadataIndex>,
    /// CRC32 checksum of footer
    pub checksum: u32,
}

impl Footer {
    /// Create new footer
    pub fn new(schema: Schema, row_count: u32) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        Footer {
            schema,
            row_group_offsets: Vec::new(),
            row_count,
            created_at: now,
            modified_at: now,
            metadata_index: None,
            checksum: 0,
        }
    }

    /// Create footer with metadata index
    pub fn with_metadata_index(schema: Schema, row_count: u32, metadata_index: MetadataIndex) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        Footer {
            schema,
            row_group_offsets: Vec::new(),
            row_count,
            created_at: now,
            modified_at: now,
            metadata_index: Some(metadata_index),
            checksum: 0,
        }
    }

    /// Serialize footer
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| {
            crate::error::Error::InvalidData(format!("Footer serialization failed: {}", e))
        })
    }

    /// Deserialize footer
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(|e| {
            crate::error::Error::InvalidData(format!("Footer deserialization failed: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FieldType, Nullability, SchemaBuilder};

    #[test]
    fn test_footer_serialization() {
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let footer = Footer::new(schema, 1000);
        let serialized = footer.serialize().unwrap();
        let deserialized = Footer::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.row_count, 1000);
        assert_eq!(deserialized.row_group_offsets.len(), 0);
    }
}
