//! Schema definitions and serialization

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Logical field type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldType {
    /// Boolean (1 bit, usually 1 byte)
    Boolean,
    /// Signed 8-bit integer
    Int8,
    /// Signed 16-bit integer
    Int16,
    /// Signed 32-bit integer
    Int32,
    /// Signed 64-bit integer
    Int64,
    /// Unsigned 8-bit integer
    UInt8,
    /// Unsigned 16-bit integer
    UInt16,
    /// Unsigned 32-bit integer
    UInt32,
    /// Unsigned 64-bit integer
    UInt64,
    /// 32-bit IEEE 754 float
    Float32,
    /// 64-bit IEEE 754 float
    Float64,
    /// Timestamp (microseconds since epoch)
    Timestamp,
    /// Date (days since 1970-01-01)
    Date,
    /// Time (microseconds since 00:00:00)
    Time,
    /// Duration (microseconds)
    Duration,
    /// UTF-8 string
    String,
    /// Enumeration (string with defined values)
    Enum,
    /// UUID (128-bit)
    Uuid,
    /// Binary blob
    Blob,
    /// Arbitrary precision decimal
    Decimal,
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldType::Boolean => write!(f, "BOOLEAN"),
            FieldType::Int8 => write!(f, "INT8"),
            FieldType::Int16 => write!(f, "INT16"),
            FieldType::Int32 => write!(f, "INT32"),
            FieldType::Int64 => write!(f, "INT64"),
            FieldType::UInt8 => write!(f, "UINT8"),
            FieldType::UInt16 => write!(f, "UINT16"),
            FieldType::UInt32 => write!(f, "UINT32"),
            FieldType::UInt64 => write!(f, "UINT64"),
            FieldType::Float32 => write!(f, "FLOAT32"),
            FieldType::Float64 => write!(f, "FLOAT64"),
            FieldType::Timestamp => write!(f, "TIMESTAMP"),
            FieldType::Date => write!(f, "DATE"),
            FieldType::Time => write!(f, "TIME"),
            FieldType::Duration => write!(f, "DURATION"),
            FieldType::String => write!(f, "STRING"),
            FieldType::Enum => write!(f, "ENUM"),
            FieldType::Uuid => write!(f, "UUID"),
            FieldType::Blob => write!(f, "BLOB"),
            FieldType::Decimal => write!(f, "DECIMAL"),
        }
    }
}

impl FieldType {
    /// Get the size in bytes for fixed-size types
    pub fn fixed_size(&self) -> Option<usize> {
        match self {
            FieldType::Boolean => Some(1),
            FieldType::Int8 => Some(1),
            FieldType::UInt8 => Some(1),
            FieldType::Int16 => Some(2),
            FieldType::UInt16 => Some(2),
            FieldType::Int32 => Some(4),
            FieldType::UInt32 => Some(4),
            FieldType::Float32 => Some(4),
            FieldType::Int64 => Some(8),
            FieldType::UInt64 => Some(8),
            FieldType::Float64 => Some(8),
            FieldType::Timestamp => Some(8),
            FieldType::Date => Some(4),
            FieldType::Time => Some(8),
            FieldType::Duration => Some(8),
            FieldType::Uuid => Some(16),
            _ => None,
        }
    }

    /// Whether this type supports null values typically
    pub fn supports_nulls(&self) -> bool {
        true
    }
}

/// Nullability modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Nullability {
    /// Field must not be null
    Required,
    /// Field may be null
    Optional,
    /// Field is a repeated array (0 or more elements)
    Repeated,
}

impl fmt::Display for Nullability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Nullability::Required => write!(f, "REQUIRED"),
            Nullability::Optional => write!(f, "OPTIONAL"),
            Nullability::Repeated => write!(f, "REPEATED"),
        }
    }
}

/// Metadata for a single field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMetadata {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Nullability
    pub nullability: Nullability,
    /// User-defined metadata key-value pairs
    pub metadata: HashMap<String, String>,
}

/// Complete schema for a columnar file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Fields in order
    pub fields: Vec<FieldMetadata>,
    /// Schema ID (deterministic hash)
    pub schema_id: u32,
}

impl Schema {
    /// Get a field by name
    pub fn field_by_name(&self, name: &str) -> Option<(usize, &FieldMetadata)> {
        self.fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.name == name)
    }

    /// Get a field by index
    pub fn field_by_index(&self, index: usize) -> Option<&FieldMetadata> {
        self.fields.get(index)
    }

    /// Number of columns
    pub fn column_count(&self) -> usize {
        self.fields.len()
    }

    /// Calculate deterministic schema hash
    pub fn calculate_id(fields: &[FieldMetadata]) -> u32 {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        for field in fields {
            hasher.update(field.name.as_bytes());
            hasher.update(&[field.field_type as u8]);
            hasher.update(&[field.nullability as u8]);
        }

        let result = hasher.finalize();
        u32::from_le_bytes([result[0], result[1], result[2], result[3]])
    }
}

/// Builder for constructing schemas
pub struct SchemaBuilder {
    fields: Vec<FieldMetadata>,
}

impl SchemaBuilder {
    /// Create a new schema builder
    pub fn new() -> Self {
        SchemaBuilder {
            fields: Vec::new(),
        }
    }

    /// Add a field to the schema
    pub fn add_field(
        mut self,
        name: impl Into<String>,
        field_type: FieldType,
        nullability: Nullability,
    ) -> Result<Self> {
        let name_str = name.into();

        // Validate field name
        if name_str.is_empty() {
            return Err(Error::InvalidSchema("Field name cannot be empty".to_string()));
        }

        // Check for duplicate names
        if self.fields.iter().any(|f| f.name == name_str) {
            return Err(Error::InvalidSchema(format!(
                "Duplicate field name: {}",
                name_str
            )));
        }

        self.fields.push(FieldMetadata {
            name: name_str,
            field_type,
            nullability,
            metadata: HashMap::new(),
        });

        Ok(self)
    }

    /// Build the schema
    pub fn build(self) -> Result<Schema> {
        if self.fields.is_empty() {
            return Err(Error::InvalidSchema(
                "Schema must have at least one field".to_string(),
            ));
        }

        let schema_id = Schema::calculate_id(&self.fields);

        Ok(Schema {
            fields: self.fields,
            schema_id,
        })
    }
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_builder() {
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("name", FieldType::String, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(schema.fields.len(), 2);
        assert_eq!(schema.fields[0].name, "id");
        assert_eq!(schema.fields[1].name, "name");
    }

    #[test]
    fn test_field_type_sizes() {
        assert_eq!(FieldType::Int64.fixed_size(), Some(8));
        assert_eq!(FieldType::Float32.fixed_size(), Some(4));
        assert_eq!(FieldType::String.fixed_size(), None);
    }

    #[test]
    fn test_deterministic_schema_id() {
        let fields1 = vec![FieldMetadata {
            name: "test".to_string(),
            field_type: FieldType::Int64,
            nullability: Nullability::Required,
            metadata: HashMap::new(),
        }];

        let id1 = Schema::calculate_id(&fields1);
        let id2 = Schema::calculate_id(&fields1);

        assert_eq!(id1, id2);
    }
}
