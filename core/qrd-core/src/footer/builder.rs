//! Footer builder - constructs deterministic footer metadata

use crate::error::Result;
use crate::footer::Footer;
use crate::schema::Schema;

/// Builder for deterministic footer construction
pub struct FooterBuilder {
    schema: Schema,
    row_count: u32,
    row_group_offsets: Vec<u64>,
    created_at: u32,
    modified_at: u32,
}

impl FooterBuilder {
    /// Create new footer builder
    pub fn new(schema: Schema, row_count: u32) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        FooterBuilder {
            schema,
            row_count,
            row_group_offsets: Vec::new(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Add row group offset
    pub fn add_row_group_offset(&mut self, offset: u64) -> &mut Self {
        self.row_group_offsets.push(offset);
        self
    }

    /// Set all row group offsets at once
    pub fn set_row_group_offsets(&mut self, offsets: Vec<u64>) -> &mut Self {
        self.row_group_offsets = offsets;
        self
    }

    /// Set creation timestamp
    pub fn set_created_at(&mut self, timestamp: u32) -> &mut Self {
        self.created_at = timestamp;
        self
    }

    /// Set modification timestamp
    pub fn set_modified_at(&mut self, timestamp: u32) -> &mut Self {
        self.modified_at = timestamp;
        self
    }

    /// Build footer with deterministic serialization
    pub fn build(self) -> Result<Footer> {
        let footer = Footer {
            schema: self.schema,
            row_group_offsets: self.row_group_offsets,
            row_count: self.row_count,
            created_at: self.created_at,
            modified_at: self.modified_at,
            metadata_index: None,
            checksum: 0, // Will be calculated by serializer
        };

        Ok(footer)
    }

    /// Build and serialize footer in deterministic order
    pub fn build_and_serialize(self) -> Result<Vec<u8>> {
        let footer = self.build()?;
        footer.serialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FieldType, Nullability, SchemaBuilder};

    fn make_schema(names: Vec<&str>) -> crate::schema::Schema {
        let mut builder = SchemaBuilder::new();
        for name in names {
            builder = builder
                .add_field(name, FieldType::Blob, Nullability::Required)
                .unwrap();
        }
        builder.build().unwrap()
    }

    #[test]
    fn test_footer_builder_basic() -> Result<()> {
        let schema = make_schema(vec!["col1"]);
        let mut builder = FooterBuilder::new(schema, 1000);

        builder.add_row_group_offset(32).add_row_group_offset(5000);

        let footer = builder.build()?;
        assert_eq!(footer.row_count, 1000);
        assert_eq!(footer.row_group_offsets.len(), 2);

        Ok(())
    }

    #[test]
    fn test_footer_builder_deterministic() -> Result<()> {
        let schema = make_schema(vec!["col1"]);

        let mut builder1 = FooterBuilder::new(schema.clone(), 1000);
        builder1
            .set_created_at(1000)
            .set_modified_at(1000)
            .add_row_group_offset(32)
            .add_row_group_offset(5000);

        let mut builder2 = FooterBuilder::new(schema, 1000);
        builder2
            .set_created_at(1000)
            .set_modified_at(1000)
            .add_row_group_offset(32)
            .add_row_group_offset(5000);

        let bytes1 = builder1.build_and_serialize()?;
        let bytes2 = builder2.build_and_serialize()?;

        assert_eq!(
            bytes1, bytes2,
            "Footer serialization should be deterministic"
        );

        Ok(())
    }
}
