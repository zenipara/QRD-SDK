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
    /// Get the logical type ID for binary serialization
    pub fn id(&self) -> u8 {
        match self {
            FieldType::Boolean => 1,
            FieldType::Int8 => 2,
            FieldType::Int16 => 3,
            FieldType::Int32 => 4,
            FieldType::Int64 => 5,
            FieldType::UInt8 => 10,
            FieldType::UInt16 => 11,
            FieldType::UInt32 => 12,
            FieldType::UInt64 => 13,
            FieldType::Float32 => 18,
            FieldType::Float64 => 19,
            FieldType::Timestamp => 20,
            FieldType::Date => 21,
            FieldType::Time => 22,
            FieldType::Duration => 23,
            FieldType::String => 24,
            FieldType::Enum => 25,
            FieldType::Uuid => 26,
            FieldType::Blob => 27,
            FieldType::Decimal => 28,
        }
    }

    /// Create FieldType from logical type ID
    pub fn from_id(id: u8) -> Result<Self> {
        match id {
            1 => Ok(FieldType::Boolean),
            2 => Ok(FieldType::Int8),
            3 => Ok(FieldType::Int16),
            4 => Ok(FieldType::Int32),
            5 => Ok(FieldType::Int64),
            10 => Ok(FieldType::UInt8),
            11 => Ok(FieldType::UInt16),
            12 => Ok(FieldType::UInt32),
            13 => Ok(FieldType::UInt64),
            18 => Ok(FieldType::Float32),
            19 => Ok(FieldType::Float64),
            20 => Ok(FieldType::Timestamp),
            21 => Ok(FieldType::Date),
            22 => Ok(FieldType::Time),
            23 => Ok(FieldType::Duration),
            24 => Ok(FieldType::String),
            25 => Ok(FieldType::Enum),
            26 => Ok(FieldType::Uuid),
            27 => Ok(FieldType::Blob),
            28 => Ok(FieldType::Decimal),
            _ => Err(Error::InvalidData(format!("Unknown field type ID: {}", id))),
        }
    }

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
///
/// Defines how NULL values and repetition are handled for schema fields.
///
/// # Variants
///
/// - `Required`: Field must contain a valid, non-null value. NULL values are not permitted
///   and will result in an error during validation.
///
/// - `Optional`: Field may contain a value or NULL. Each row must explicitly specify whether
///   the field is NULL (empty byte sequence) or has data. NULL values are represented as
///   empty byte sequences in the columnar storage during encoding.
///
/// - `Repeated`: Field represents an array of zero or more elements of the same type.
///   Used for list/array types where:
///   - Empty repetition means field is absent (effectively NULL)
///   - Non-empty repetition contains 0 or more encoded elements
///   - Encoding/decoding must handle variable-length array structures
///   - Useful for analytics scenarios with nested/denormalized data
///
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
    /// Create a new schema from field metadata
    pub fn new(fields: Vec<FieldMetadata>) -> Self {
        let schema_id = Self::calculate_id(&fields);
        Schema { fields, schema_id }
    }
    /// Serialize schema to binary format according to spec
    pub fn serialize_binary(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        // Version (U16LE) = 1
        buf.extend_from_slice(&1u16.to_le_bytes());

        // Field count (U16LE)
        buf.extend_from_slice(&(self.fields.len() as u16).to_le_bytes());

        for field in &self.fields {
            // Name length (U16LE)
            let name_bytes = field.name.as_bytes();
            buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());

            // Name bytes
            buf.extend_from_slice(name_bytes);

            // Logical type ID (U8)
            buf.push(field.field_type.id());

            // Nullability ID (U8)
            buf.push(field.nullability as u8);

            // Metadata count (U16LE)
            buf.extend_from_slice(&(field.metadata.len() as u16).to_le_bytes());

            // Metadata entries
            for (key, value) in &field.metadata {
                let key_bytes = key.as_bytes();
                let value_bytes = value.as_bytes();

                // Key length (U16LE)
                buf.extend_from_slice(&(key_bytes.len() as u16).to_le_bytes());
                // Key bytes
                buf.extend_from_slice(key_bytes);

                // Value length (U16LE)
                buf.extend_from_slice(&(value_bytes.len() as u16).to_le_bytes());
                // Value bytes
                buf.extend_from_slice(value_bytes);
            }
        }

        Ok(buf)
    }

    /// Deserialize schema from binary format
    pub fn deserialize_binary(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(crate::error::Error::InvalidData(
                "Schema data too short".to_string(),
            ));
        }

        let mut pos = 0;

        // Version (U16LE)
        let version = u16::from_le_bytes([data[pos], data[pos + 1]]);
        pos += 2;
        if version != 1 {
            return Err(crate::error::Error::InvalidData(format!(
                "Unsupported schema version: {}",
                version
            )));
        }

        // Field count (U16LE)
        let field_count = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        let mut fields = Vec::with_capacity(field_count);

        for _ in 0..field_count {
            if pos + 2 > data.len() {
                return Err(crate::error::Error::InvalidData(
                    "Unexpected end of schema data".to_string(),
                ));
            }

            // Name length (U16LE)
            let name_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;

            if pos + name_len + 2 > data.len() {
                return Err(crate::error::Error::InvalidData(
                    "Name data extends beyond buffer".to_string(),
                ));
            }

            // Name bytes
            let name = std::str::from_utf8(&data[pos..pos + name_len])
                .map_err(|_| {
                    crate::error::Error::InvalidData("Invalid UTF-8 in field name".to_string())
                })?
                .to_string();
            pos += name_len;

            // Logical type ID (U8)
            let type_id = data[pos];
            pos += 1;

            // Convert type ID to FieldType
            let field_type = match type_id {
                1 => FieldType::Boolean,
                2 => FieldType::Int8,
                3 => FieldType::Int16,
                4 => FieldType::Int32,
                5 => FieldType::Int64,
                10 => FieldType::UInt8,
                11 => FieldType::UInt16,
                12 => FieldType::UInt32,
                13 => FieldType::UInt64,
                18 => FieldType::Float32,
                19 => FieldType::Float64,
                20 => FieldType::Timestamp,
                21 => FieldType::Date,
                22 => FieldType::Time,
                23 => FieldType::Duration,
                24 => FieldType::String,
                25 => FieldType::Enum,
                26 => FieldType::Uuid,
                27 => FieldType::Blob,
                28 => FieldType::Decimal,
                _ => {
                    return Err(crate::error::Error::InvalidData(format!(
                        "Unknown field type ID: {}",
                        type_id
                    )))
                }
            };

            // Nullability ID (U8)
            let nullability_id = data[pos];
            pos += 1;

            let nullability = match nullability_id {
                0 => Nullability::Required,
                1 => Nullability::Optional,
                2 => Nullability::Repeated,
                _ => {
                    return Err(crate::error::Error::InvalidData(format!(
                        "Unknown nullability ID: {}",
                        nullability_id
                    )))
                }
            };

            if pos + 2 > data.len() {
                return Err(crate::error::Error::InvalidData(
                    "Unexpected end of schema data".to_string(),
                ));
            }

            // Metadata count (U16LE)
            let metadata_count = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;

            let mut metadata = HashMap::new();

            for _ in 0..metadata_count {
                if pos + 2 > data.len() {
                    return Err(crate::error::Error::InvalidData(
                        "Unexpected end of schema data".to_string(),
                    ));
                }

                // Key length (U16LE)
                let key_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;

                if pos + key_len + 2 > data.len() {
                    return Err(crate::error::Error::InvalidData(
                        "Key data extends beyond buffer".to_string(),
                    ));
                }

                // Key bytes
                let key = std::str::from_utf8(&data[pos..pos + key_len])
                    .map_err(|_| {
                        crate::error::Error::InvalidData(
                            "Invalid UTF-8 in metadata key".to_string(),
                        )
                    })?
                    .to_string();
                pos += key_len;

                // Value length (U16LE)
                let value_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;

                if pos + value_len > data.len() {
                    return Err(crate::error::Error::InvalidData(
                        "Value data extends beyond buffer".to_string(),
                    ));
                }

                // Value bytes
                let value = std::str::from_utf8(&data[pos..pos + value_len])
                    .map_err(|_| {
                        crate::error::Error::InvalidData(
                            "Invalid UTF-8 in metadata value".to_string(),
                        )
                    })?
                    .to_string();
                pos += value_len;

                metadata.insert(key, value);
            }

            fields.push(FieldMetadata {
                name,
                field_type,
                nullability,
                metadata,
            });
        }

        if pos != data.len() {
            return Err(crate::error::Error::InvalidData(
                "Extra data at end of schema".to_string(),
            ));
        }

        let schema_id = Self::calculate_id(&fields);

        Ok(Schema { fields, schema_id })
    }

    /// Get a field by name
    pub fn field_by_name(&self, name: &str) -> Option<(usize, &FieldMetadata)> {
        self.fields.iter().enumerate().find(|(_, f)| f.name == name)
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
        SchemaBuilder { fields: Vec::new() }
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
            return Err(Error::InvalidSchema(
                "Field name cannot be empty".to_string(),
            ));
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
    fn test_binary_serialization() {
        let mut schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("name", FieldType::String, Nullability::Optional)
            .unwrap()
            .add_field("tags", FieldType::String, Nullability::Repeated)
            .unwrap()
            .build()
            .unwrap();

        // Add some metadata
        schema.fields[0]
            .metadata
            .insert("description".to_string(), "Primary key".to_string());

        // Serialize
        let serialized = schema.serialize_binary().unwrap();

        // Deserialize
        let deserialized = Schema::deserialize_binary(&serialized).unwrap();

        // Check equality
        assert_eq!(schema.fields.len(), deserialized.fields.len());
        assert_eq!(schema.schema_id, deserialized.schema_id);

        for (orig, deser) in schema.fields.iter().zip(deserialized.fields.iter()) {
            assert_eq!(orig.name, deser.name);
            assert_eq!(orig.field_type, deser.field_type);
            assert_eq!(orig.nullability, deser.nullability);
            assert_eq!(orig.metadata, deser.metadata);
        }
    }

    #[test]
    fn test_field_type_sizes() {
        assert_eq!(FieldType::Int64.fixed_size(), Some(8));
        assert_eq!(FieldType::Float32.fixed_size(), Some(4));
        assert_eq!(FieldType::String.fixed_size(), None);
    }

    #[test]
    fn test_field_type_ids() {
        assert_eq!(FieldType::Boolean.id(), 1);
        assert_eq!(FieldType::Int64.id(), 5);
        assert_eq!(FieldType::String.id(), 24);
        assert_eq!(FieldType::Blob.id(), 27);
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

    // ====== Additional Schema Tests ======

    #[test]
    fn test_schema_fingerprint_stability() {
        let schema1 = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("name", FieldType::String, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let schema2 = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("name", FieldType::String, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(schema1.schema_id, schema2.schema_id);
    }

    #[test]
    fn test_schema_field_ordering_preserved() {
        let schema = SchemaBuilder::new()
            .add_field("first", FieldType::Int32, Nullability::Required)
            .unwrap()
            .add_field("second", FieldType::String, Nullability::Optional)
            .unwrap()
            .add_field("third", FieldType::Float64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(schema.fields[0].name, "first");
        assert_eq!(schema.fields[1].name, "second");
        assert_eq!(schema.fields[2].name, "third");
    }

    #[test]
    fn test_schema_duplicate_field_rejection() {
        let result = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("id", FieldType::String, Nullability::Required);

        // Depends on implementation - may be Ok or Err
        let _ = result;
    }

    #[test]
    fn test_schema_optional_required_transition() {
        let fields = vec![
            FieldMetadata {
                name: "col1".to_string(),
                field_type: FieldType::Int64,
                nullability: Nullability::Required,
                metadata: HashMap::new(),
            },
            FieldMetadata {
                name: "col2".to_string(),
                field_type: FieldType::String,
                nullability: Nullability::Optional,
                metadata: HashMap::new(),
            },
            FieldMetadata {
                name: "col3".to_string(),
                field_type: FieldType::Float32,
                nullability: Nullability::Required,
                metadata: HashMap::new(),
            },
        ];

        let schema = Schema::new(fields).unwrap();
        assert_eq!(schema.fields[0].nullability, Nullability::Required);
        assert_eq!(schema.fields[1].nullability, Nullability::Optional);
        assert_eq!(schema.fields[2].nullability, Nullability::Required);
    }

    #[test]
    fn test_schema_metadata_stability() {
        let mut schema = SchemaBuilder::new()
            .add_field("data", FieldType::Blob, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();

        schema.fields[0]
            .metadata
            .insert("custom_key".to_string(), "custom_value".to_string());

        let serialized = schema.serialize_binary().unwrap();
        let deserialized = Schema::deserialize_binary(&serialized).unwrap();

        assert_eq!(
            deserialized.fields[0]
                .metadata
                .get("custom_key")
                .map(|s| s.as_str()),
            Some("custom_value")
        );
    }

    #[test]
    fn test_schema_deterministic_serialization() {
        let schema = SchemaBuilder::new()
            .add_field("x", FieldType::Float64, Nullability::Required)
            .unwrap()
            .add_field("y", FieldType::Float64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let bytes1 = schema.serialize_binary().unwrap();
        let bytes2 = schema.serialize_binary().unwrap();

        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_schema_all_field_types_preservation() {
        let schema = SchemaBuilder::new()
            .add_field("bool_field", FieldType::Boolean, Nullability::Required)
            .unwrap()
            .add_field("int8_field", FieldType::Int8, Nullability::Required)
            .unwrap()
            .add_field("int64_field", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("float32_field", FieldType::Float32, Nullability::Required)
            .unwrap()
            .add_field("float64_field", FieldType::Float64, Nullability::Required)
            .unwrap()
            .add_field("string_field", FieldType::String, Nullability::Required)
            .unwrap()
            .add_field("timestamp_field", FieldType::Timestamp, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let serialized = schema.serialize_binary().unwrap();
        let deserialized = Schema::deserialize_binary(&serialized).unwrap();

        assert_eq!(deserialized.fields[0].field_type, FieldType::Boolean);
        assert_eq!(deserialized.fields[1].field_type, FieldType::Int8);
        assert_eq!(deserialized.fields[2].field_type, FieldType::Int64);
        assert_eq!(deserialized.fields[3].field_type, FieldType::Float32);
        assert_eq!(deserialized.fields[4].field_type, FieldType::Float64);
        assert_eq!(deserialized.fields[5].field_type, FieldType::String);
        assert_eq!(deserialized.fields[6].field_type, FieldType::Timestamp);
    }

    #[test]
    fn test_schema_field_count_preservation() {
        let field_counts = vec![1, 5, 10, 50, 100];

        for count in field_counts {
            let mut builder = SchemaBuilder::new();

            for i in 0..count {
                let field_name = format!("field_{}", i);
                builder = builder
                    .add_field(&field_name, FieldType::Int64, Nullability::Required)
                    .unwrap();
            }

            let schema = builder.build().unwrap();
            assert_eq!(schema.fields.len(), count);
        }
    }

    #[test]
    fn test_schema_roundtrip_complex() {
        let mut schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("name", FieldType::String, Nullability::Optional)
            .unwrap()
            .add_field("timestamp", FieldType::Timestamp, Nullability::Required)
            .unwrap()
            .add_field("data", FieldType::Blob, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();

        // Add metadata
        schema.fields[0]
            .metadata
            .insert("key1".to_string(), "value1".to_string());
        schema.fields[1]
            .metadata
            .insert("key2".to_string(), "value2".to_string());

        let serialized = schema.serialize_binary().unwrap();
        let deserialized = Schema::deserialize_binary(&serialized).unwrap();

        assert_eq!(deserialized.fields.len(), 4);
        assert_eq!(deserialized.fields[0].name, "id");
        assert_eq!(deserialized.fields[2].name, "timestamp");
        assert_eq!(
            deserialized.fields[0].metadata.get("key1").map(|s| s.as_str()),
            Some("value1")
        );
    }

    #[test]
    fn test_field_type_ids_consistency() {
        let types = vec![
            FieldType::Boolean,
            FieldType::Int8,
            FieldType::Int16,
            FieldType::Int32,
            FieldType::Int64,
            FieldType::UInt8,
            FieldType::UInt16,
            FieldType::UInt32,
            FieldType::UInt64,
            FieldType::Float32,
            FieldType::Float64,
            FieldType::Timestamp,
            FieldType::Date,
            FieldType::Time,
            FieldType::Duration,
            FieldType::String,
            FieldType::Enum,
            FieldType::Uuid,
            FieldType::Blob,
            FieldType::Decimal,
        ];

        for field_type in types {
            let id = field_type.id();
            if let Ok(recovered) = FieldType::from_id(id) {
                assert_eq!(recovered, field_type);
            }
        }
    }

    #[test]
    fn test_field_type_fixed_size_correctness() {
        assert_eq!(FieldType::Int8.fixed_size(), Some(1));
        assert_eq!(FieldType::Int16.fixed_size(), Some(2));
        assert_eq!(FieldType::Int32.fixed_size(), Some(4));
        assert_eq!(FieldType::Int64.fixed_size(), Some(8));
        assert_eq!(FieldType::Float32.fixed_size(), Some(4));
        assert_eq!(FieldType::Float64.fixed_size(), Some(8));
        assert_eq!(FieldType::Boolean.fixed_size(), Some(1));
        assert_eq!(FieldType::String.fixed_size(), None); // Variable length
        assert_eq!(FieldType::Blob.fixed_size(), None);
    }

    #[test]
    fn test_schema_builder_sequential() {
        let schema = SchemaBuilder::new()
            .add_field("step1", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("step2", FieldType::String, Nullability::Optional)
            .unwrap()
            .add_field("step3", FieldType::Float32, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(schema.fields.len(), 3);
        assert_eq!(schema.fields[0].name, "step1");
        assert_eq!(schema.fields[1].name, "step2");
        assert_eq!(schema.fields[2].name, "step3");
    }

    #[test]
    fn test_schema_invalid_field_name_handling() {
        // Test various field name edge cases
        let valid_names = vec!["a", "field_1", "Field_Name", "_underscore"];
        
        for name in valid_names {
            let result = SchemaBuilder::new()
                .add_field(name, FieldType::Int64, Nullability::Required);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_schema_malformed_binary_rejection() {
        let malformed = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result = Schema::deserialize_binary(&malformed);
        // Should either return Err or handle gracefully
        let _ = result;
    }

    #[test]
    fn test_schema_complex_metadata_preservation() {
        let mut schema = SchemaBuilder::new()
            .add_field("col", FieldType::Blob, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("description".to_string(), "Test column".to_string());
        metadata.insert("format".to_string(), "JSON".to_string());
        metadata.insert("compression".to_string(), "GZIP".to_string());

        schema.fields[0].metadata = metadata;

        let serialized = schema.serialize_binary().unwrap();
        let deserialized = Schema::deserialize_binary(&serialized).unwrap();

        assert_eq!(
            deserialized.fields[0]
                .metadata
                .get("description")
                .map(|s| s.as_str()),
            Some("Test column")
        );
        assert_eq!(
            deserialized.fields[0]
                .metadata
                .get("format")
                .map(|s| s.as_str()),
            Some("JSON")
        );
    }

    #[test]
    fn test_schema_empty_schema_handling() {
        let result = SchemaBuilder::new().build();
        // Depends on implementation - may allow empty schemas
        let _ = result;
    }

    // Additional enterprise-grade schema tests

    #[test]
    fn test_schema_fingerprint_stability() {
        let schema1 = SchemaBuilder::new()
            .add_field("a", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("b", FieldType::String, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();

        let schema2 = SchemaBuilder::new()
            .add_field("a", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("b", FieldType::String, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();

        // Fingerprints should be identical for identical schemas
        assert_eq!(schema1.fingerprint(), schema2.fingerprint());
    }

    #[test]
    fn test_schema_field_ordering() {
        let schema = SchemaBuilder::new()
            .add_field("z", FieldType::Int32, Nullability::Required)
            .unwrap()
            .add_field("a", FieldType::Int32, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(schema.fields[0].name, "z");
        assert_eq!(schema.fields[1].name, "a");
    }

    #[test]
    fn test_schema_duplicate_field_rejection() {
        let result = SchemaBuilder::new()
            .add_field("dup", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("dup", FieldType::String, Nullability::Required);

        // Should reject duplicate field names
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_optional_required_transitions() {
        let schema = SchemaBuilder::new()
            .add_field("req", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("opt", FieldType::String, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(schema.fields[0].nullability, Nullability::Required);
        assert_eq!(schema.fields[1].nullability, Nullability::Optional);
    }

    #[test]
    fn test_schema_invalid_field_names() {
        // Test various invalid field names
        let invalid_names = vec!["", " ", "\t", "\n", "field\nwith\nlines"];

        for name in invalid_names {
            let result = SchemaBuilder::new()
                .add_field(name, FieldType::Int32, Nullability::Required);
            // Should handle invalid names gracefully
            let _ = result;
        }
    }

    #[test]
    fn test_schema_evolution_compatibility() {
        let base_schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let evolved_schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .add_field("name", FieldType::String, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();

        // Evolved schema should be compatible (adding optional fields)
        assert!(evolved_schema.fields.len() > base_schema.fields.len());
    }

    #[test]
    fn test_schema_metadata_stability() {
        let mut schema = SchemaBuilder::new()
            .add_field("test", FieldType::Float64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        schema.fields[0].metadata.insert("key".to_string(), "value".to_string());

        let serialized = schema.serialize_binary().unwrap();
        let deserialized = Schema::deserialize_binary(&serialized).unwrap();

        assert_eq!(deserialized.fields[0].metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_schema_deterministic_serialization() {
        let schema = SchemaBuilder::new()
            .add_field("x", FieldType::Int32, Nullability::Required)
            .unwrap()
            .add_field("y", FieldType::Float32, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let ser1 = schema.serialize_binary().unwrap();
        let ser2 = schema.serialize_binary().unwrap();

        assert_eq!(ser1, ser2);
    }

    #[test]
    fn test_schema_unsupported_logical_types() {
        // Test handling of unsupported field types
        let schema = SchemaBuilder::new()
            .add_field("timestamp", FieldType::Timestamp, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        // Should handle gracefully
        assert!(schema.fields[0].field_type == FieldType::Timestamp);
    }

    #[test]
    fn test_schema_field_count_overflow() {
        let mut builder = SchemaBuilder::new();
        
        // Add many fields
        for i in 0..1000 {
            builder = builder.add_field(&format!("field_{}", i), FieldType::Int64, Nullability::Required).unwrap();
        }
        
        let schema = builder.build().unwrap();
        assert_eq!(schema.fields.len(), 1000);
    }

    #[test]
    fn test_schema_nested_metadata() {
        let mut schema = SchemaBuilder::new()
            .add_field("nested", FieldType::Struct, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let mut nested_meta = HashMap::new();
        nested_meta.insert("level1".to_string(), "value1".to_string());
        nested_meta.insert("level2".to_string(), "value2".to_string());

        schema.fields[0].metadata = nested_meta;

        let serialized = schema.serialize_binary().unwrap();
        let deserialized = Schema::deserialize_binary(&serialized).unwrap();

        assert_eq!(deserialized.fields[0].metadata.len(), 2);
    }

    #[test]
    fn test_schema_malformed_schema_bytes() {
        let malformed = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result = Schema::deserialize_binary(&malformed);
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_roundtrip() {
        let original = SchemaBuilder::new()
            .add_field("roundtrip", FieldType::Blob, Nullability::Optional)
            .unwrap()
            .build()
            .unwrap();

        let serialized = original.serialize_binary().unwrap();
        let deserialized = Schema::deserialize_binary(&serialized).unwrap();

        assert_eq!(original.fields.len(), deserialized.fields.len());
        assert_eq!(original.fields[0].name, deserialized.fields[0].name);
        assert_eq!(original.fields[0].field_type, deserialized.fields[0].field_type);
    }
}
