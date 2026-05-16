//! Schema Evolution: Support for field rename and type change
//!
//! This module provides safe schema evolution with:
//! - Field rename mapping with validation
//! - Type change with compatibility checking
//! - Forward/backward compatibility rules
//! - Migration rules and constraints

use super::{FieldMetadata, FieldType, Nullability, Schema};
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type compatibility for automatic conversions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeCompatibility {
    /// Types are identical
    Identical,
    /// Lossless conversion (e.g., Int32 -> Int64)
    Widening,
    /// Potentially lossy conversion (e.g., Float64 -> Int32)
    Narrowing,
    /// Incompatible types, requires explicit conversion
    Incompatible,
}

impl TypeCompatibility {
    /// Check if conversion is safe (non-lossy)
    pub fn is_safe(&self) -> bool {
        matches!(self, TypeCompatibility::Identical | TypeCompatibility::Widening)
    }

    /// Check if conversion is possible at all
    pub fn is_possible(&self) -> bool {
        matches!(
            self,
            TypeCompatibility::Identical | TypeCompatibility::Widening | TypeCompatibility::Narrowing
        )
    }
}

/// Field mapping for schema evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Original field name in old schema
    pub old_name: String,
    /// New field name in new schema
    pub new_name: String,
    /// Old field type
    pub old_type: FieldType,
    /// New field type
    pub new_type: FieldType,
    /// Type compatibility
    pub compatibility: TypeCompatibility,
    /// Optional conversion function name
    pub converter: Option<String>,
    /// Whether nullability changed
    pub nullability_changed: bool,
}

/// Schema evolution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMigration {
    /// Old schema ID
    pub old_schema_id: u32,
    /// New schema ID
    pub new_schema_id: u32,
    /// Field mappings (old field index -> mapping)
    pub field_mappings: Vec<Option<FieldMapping>>,
    /// New fields that don't exist in old schema
    pub new_fields: Vec<FieldMetadata>,
    /// Fields being removed (old indices)
    pub removed_field_indices: Vec<usize>,
    /// Is this a breaking change?
    pub is_breaking: bool,
    /// Migration version for tracking
    pub migration_version: u32,
}

/// Conversion specification for type changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeConversion {
    /// Source type
    pub from_type: FieldType,
    /// Target type
    pub to_type: FieldType,
    /// Conversion name (e.g., "string_to_int", "float_to_timestamp")
    pub name: String,
    /// Description
    pub description: String,
    /// Is this a safe conversion?
    pub is_safe: bool,
}

/// Schema evolution engine
pub struct SchemaEvolution;

impl SchemaEvolution {
    /// Get type compatibility between two types
    pub fn check_type_compatibility(from: FieldType, to: FieldType) -> TypeCompatibility {
        use FieldType::*;

        match (from, to) {
            // Identical
            (a, b) if a == b => TypeCompatibility::Identical,

            // Numeric widening (safe)
            (Int8, Int16 | Int32 | Int64) => TypeCompatibility::Widening,
            (Int16, Int32 | Int64) => TypeCompatibility::Widening,
            (Int32, Int64) => TypeCompatibility::Widening,
            (UInt8, UInt16 | UInt32 | UInt64) => TypeCompatibility::Widening,
            (UInt16, UInt32 | UInt64) => TypeCompatibility::Widening,
            (UInt32, UInt64) => TypeCompatibility::Widening,
            (Float32, Float64) => TypeCompatibility::Widening,

            // Integer to float (widening in range, not identical)
            (Int8 | UInt8, Float32 | Float64) => TypeCompatibility::Widening,
            (Int16 | UInt16, Float32 | Float64) => TypeCompatibility::Widening,
            (Int32 | UInt32, Float64) => TypeCompatibility::Widening,

            // Numeric narrowing (potentially lossy)
            // Int8 narrowing: cannot narrow further
            (Int16, Int8) => TypeCompatibility::Narrowing,
            (Int32, Int8 | Int16) => TypeCompatibility::Narrowing,
            (Int64, Int8 | Int16 | Int32) => TypeCompatibility::Narrowing,
            (UInt8, Int8) => TypeCompatibility::Narrowing,
            (UInt16, Int8 | Int16 | UInt8) => TypeCompatibility::Narrowing,
            (UInt32, Int8 | Int16 | Int32 | UInt8 | UInt16) => TypeCompatibility::Narrowing,
            (UInt64, Int8 | Int16 | Int32 | Int64 | UInt8 | UInt16 | UInt32) => {
                TypeCompatibility::Narrowing
            }
            (Float64, Float32) => TypeCompatibility::Narrowing,
            (Float64, Int64 | Int32 | Int16 | Int8) => TypeCompatibility::Narrowing,
            (Float64, UInt64 | UInt32 | UInt16 | UInt8) => TypeCompatibility::Narrowing,
            (Float32, Int32 | Int16 | Int8) => TypeCompatibility::Narrowing,
            (Float32, UInt32 | UInt16 | UInt8) => TypeCompatibility::Narrowing,

            // String conversions
            (String, Uuid) => TypeCompatibility::Narrowing, // String to UUID (if valid)
            (Uuid, String) => TypeCompatibility::Widening,  // UUID to String (always valid)

            // Blob conversions
            (Blob, String) => TypeCompatibility::Narrowing, // Binary to String (if valid UTF-8)
            (String, Blob) => TypeCompatibility::Widening,  // String to Blob (always valid)

            // Temporal conversions
            (Timestamp, String) => TypeCompatibility::Widening, // Timestamp serializes to string
            (Date, String) => TypeCompatibility::Widening,
            (Time, String) => TypeCompatibility::Widening,
            (String, Timestamp) => TypeCompatibility::Narrowing, // String parse to timestamp
            (String, Date) => TypeCompatibility::Narrowing,
            (String, Time) => TypeCompatibility::Narrowing,

            // Everything else is incompatible
            _ => TypeCompatibility::Incompatible,
        }
    }

    /// Validate if a field can be renamed
    pub fn validate_field_rename(
        old_field: &FieldMetadata,
        new_field: &FieldMetadata,
    ) -> Result<()> {
        // Type must be identical for rename-only
        if old_field.field_type != new_field.field_type {
            return Err(Error::SchemaError(
                format!(
                    "Cannot rename field with type change: {} ({}) -> {} ({})",
                    old_field.name,
                    old_field.field_type,
                    new_field.name,
                    new_field.field_type
                )
            ));
        }

        // Nullability should ideally remain the same
        if old_field.nullability != new_field.nullability {
            return Err(Error::SchemaError(
                format!(
                    "Cannot rename field with nullability change: {} ({}) -> {} ({})",
                    old_field.name,
                    old_field.nullability,
                    new_field.name,
                    new_field.nullability
                )
            ));
        }

        Ok(())
    }

    /// Validate type change compatibility
    pub fn validate_type_change(
        old_type: FieldType,
        new_type: FieldType,
        allow_breaking: bool,
    ) -> Result<TypeCompatibility> {
        let compatibility = Self::check_type_compatibility(old_type, new_type);

        match compatibility {
            TypeCompatibility::Incompatible => Err(Error::SchemaError(
                format!(
                    "Incompatible type change: {} -> {}",
                    old_type, new_type
                )
            )),
            TypeCompatibility::Narrowing if !allow_breaking => Err(Error::SchemaError(
                format!(
                    "Lossy type change not allowed: {} -> {} (requires explicit approval)",
                    old_type, new_type
                )
            )),
            _ => Ok(compatibility),
        }
    }

    /// Plan a schema migration from old to new schema
    pub fn plan_migration(old_schema: &Schema, new_schema: &Schema) -> Result<SchemaMigration> {
        let mut field_mappings = vec![None; old_schema.fields.len()];
        let mut new_fields = Vec::new();
        let mut removed_field_indices = Vec::new();
        let mut is_breaking = false;

        // Track which new fields have been mapped
        let mut mapped_new_indices = std::collections::HashSet::new();

        // Try to match fields by name first
        for (old_idx, old_field) in old_schema.fields.iter().enumerate() {
            if let Some((new_idx, new_field)) = new_schema.field_by_name(&old_field.name) {
                mapped_new_indices.insert(new_idx);
                let compatibility =
                    Self::check_type_compatibility(old_field.field_type, new_field.field_type);

                if compatibility == TypeCompatibility::Incompatible {
                    is_breaking = true;
                }

                field_mappings[old_idx] = Some(FieldMapping {
                    old_name: old_field.name.clone(),
                    new_name: new_field.name.clone(),
                    old_type: old_field.field_type,
                    new_type: new_field.field_type,
                    compatibility,
                    converter: None,
                    nullability_changed: old_field.nullability != new_field.nullability,
                });
            } else {
                // Field removed
                removed_field_indices.push(old_idx);
                if old_field.nullability == Nullability::Required {
                    is_breaking = true;
                }
            }
        }

        // Find new fields
        for (new_idx, new_field) in new_schema.fields.iter().enumerate() {
            if !mapped_new_indices.contains(&new_idx) {
                new_fields.push(new_field.clone());
                // Adding required fields is breaking
                if new_field.nullability == Nullability::Required {
                    is_breaking = true;
                }
            }
        }

        Ok(SchemaMigration {
            old_schema_id: old_schema.schema_id,
            new_schema_id: new_schema.schema_id,
            field_mappings,
            new_fields,
            removed_field_indices,
            is_breaking,
            migration_version: 1,
        })
    }

    /// Check if migration is forward compatible (old reader can read new data)
    pub fn is_forward_compatible(migration: &SchemaMigration) -> bool {
        // Forward compatible if:
        // 1. No required fields are removed
        // 2. No types become incompatible

        for mapping in &migration.field_mappings {
            if let Some(m) = mapping {
                if m.compatibility == TypeCompatibility::Incompatible {
                    return false;
                }
            }
        }

        true
    }

    /// Check if migration is backward compatible (new reader can read old data)
    pub fn is_backward_compatible(migration: &SchemaMigration) -> bool {
        // Backward compatible if:
        // 1. No required fields are added
        // 2. No types are removed or become incompatible

        for new_field in &migration.new_fields {
            if new_field.nullability == Nullability::Required {
                return false;
            }
        }

        for mapping in &migration.field_mappings {
            if let Some(m) = mapping {
                if m.compatibility == TypeCompatibility::Incompatible {
                    return false;
                }
            }
        }

        true
    }

    /// Get all supported type conversions
    pub fn get_type_conversions() -> Vec<TypeConversion> {
        vec![
            TypeConversion {
                from_type: FieldType::Int32,
                to_type: FieldType::Int64,
                name: "int32_to_int64".to_string(),
                description: "Widen 32-bit signed integer to 64-bit".to_string(),
                is_safe: true,
            },
            TypeConversion {
                from_type: FieldType::Int64,
                to_type: FieldType::Int32,
                name: "int64_to_int32".to_string(),
                description: "Narrow 64-bit signed integer to 32-bit (may lose data)".to_string(),
                is_safe: false,
            },
            TypeConversion {
                from_type: FieldType::Float32,
                to_type: FieldType::Float64,
                name: "float32_to_float64".to_string(),
                description: "Widen 32-bit float to 64-bit".to_string(),
                is_safe: true,
            },
            TypeConversion {
                from_type: FieldType::String,
                to_type: FieldType::Uuid,
                name: "string_to_uuid".to_string(),
                description: "Parse string as UUID (must be valid UUID format)".to_string(),
                is_safe: false,
            },
            TypeConversion {
                from_type: FieldType::Uuid,
                to_type: FieldType::String,
                name: "uuid_to_string".to_string(),
                description: "Convert UUID to string representation".to_string(),
                is_safe: true,
            },
            TypeConversion {
                from_type: FieldType::String,
                to_type: FieldType::Blob,
                name: "string_to_blob".to_string(),
                description: "Encode string as UTF-8 blob".to_string(),
                is_safe: true,
            },
            TypeConversion {
                from_type: FieldType::Blob,
                to_type: FieldType::String,
                name: "blob_to_string".to_string(),
                description: "Decode blob as UTF-8 string (must be valid UTF-8)".to_string(),
                is_safe: false,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FieldMetadata, Nullability};

    #[test]
    fn test_type_compatibility_identical() {
        let compat = SchemaEvolution::check_type_compatibility(FieldType::Int64, FieldType::Int64);
        assert_eq!(compat, TypeCompatibility::Identical);
    }

    #[test]
    fn test_type_compatibility_widening() {
        let compat = SchemaEvolution::check_type_compatibility(FieldType::Int32, FieldType::Int64);
        assert_eq!(compat, TypeCompatibility::Widening);
    }

    #[test]
    fn test_type_compatibility_narrowing() {
        let compat = SchemaEvolution::check_type_compatibility(FieldType::Int64, FieldType::Int32);
        assert_eq!(compat, TypeCompatibility::Narrowing);
    }

    #[test]
    fn test_type_compatibility_incompatible() {
        let compat =
            SchemaEvolution::check_type_compatibility(FieldType::Int64, FieldType::Boolean);
        assert_eq!(compat, TypeCompatibility::Incompatible);
    }

    #[test]
    fn test_field_rename_validation() {
        let old_field = FieldMetadata {
            name: "user_id".to_string(),
            field_type: FieldType::Int64,
            nullability: Nullability::Required,
            metadata: HashMap::new(),
        };

        let new_field = FieldMetadata {
            name: "id".to_string(),
            field_type: FieldType::Int64,
            nullability: Nullability::Required,
            metadata: HashMap::new(),
        };

        // Should succeed
        assert!(SchemaEvolution::validate_field_rename(&old_field, &new_field).is_ok());
    }

    #[test]
    fn test_field_rename_with_type_change_fails() {
        let old_field = FieldMetadata {
            name: "user_id".to_string(),
            field_type: FieldType::Int64,
            nullability: Nullability::Required,
            metadata: HashMap::new(),
        };

        let new_field = FieldMetadata {
            name: "id".to_string(),
            field_type: FieldType::String,
            nullability: Nullability::Required,
            metadata: HashMap::new(),
        };

        // Should fail
        assert!(SchemaEvolution::validate_field_rename(&old_field, &new_field).is_err());
    }

    #[test]
    fn test_migration_planning() {
        let old_schema = Schema::new(vec![
            FieldMetadata {
                name: "id".to_string(),
                field_type: FieldType::Int64,
                nullability: Nullability::Required,
                metadata: HashMap::new(),
            },
            FieldMetadata {
                name: "name".to_string(),
                field_type: FieldType::String,
                nullability: Nullability::Optional,
                metadata: HashMap::new(),
            },
        ]);

        let new_schema = Schema::new(vec![
            FieldMetadata {
                name: "id".to_string(),
                field_type: FieldType::Int64,
                nullability: Nullability::Required,
                metadata: HashMap::new(),
            },
            FieldMetadata {
                name: "name".to_string(),
                field_type: FieldType::String,
                nullability: Nullability::Optional,
                metadata: HashMap::new(),
            },
            FieldMetadata {
                name: "created_at".to_string(),
                field_type: FieldType::Timestamp,
                nullability: Nullability::Optional,
                metadata: HashMap::new(),
            },
        ]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        // Check field mappings
        assert_eq!(migration.field_mappings[0].as_ref().unwrap().old_name, "id");
        assert_eq!(migration.field_mappings[1].as_ref().unwrap().old_name, "name");

        // Check new fields
        assert_eq!(migration.new_fields.len(), 1);
        assert_eq!(migration.new_fields[0].name, "created_at");
    }

    #[test]
    fn test_backward_compatibility() {
        let old_schema = Schema::new(vec![FieldMetadata {
            name: "id".to_string(),
            field_type: FieldType::Int64,
            nullability: Nullability::Required,
            metadata: HashMap::new(),
        }]);

        let new_schema = Schema::new(vec![
            FieldMetadata {
                name: "id".to_string(),
                field_type: FieldType::Int64,
                nullability: Nullability::Required,
                metadata: HashMap::new(),
            },
            FieldMetadata {
                name: "timestamp".to_string(),
                field_type: FieldType::Timestamp,
                nullability: Nullability::Optional,
                metadata: HashMap::new(),
            },
        ]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        // Adding optional field should be backward compatible
        assert!(!migration.is_breaking);
        assert!(SchemaEvolution::is_backward_compatible(&migration));
    }

    #[test]
    fn test_breaking_change_required_field() {
        let old_schema = Schema::new(vec![FieldMetadata {
            name: "id".to_string(),
            field_type: FieldType::Int64,
            nullability: Nullability::Required,
            metadata: HashMap::new(),
        }]);

        let new_schema = Schema::new(vec![
            FieldMetadata {
                name: "id".to_string(),
                field_type: FieldType::Int64,
                nullability: Nullability::Required,
                metadata: HashMap::new(),
            },
            FieldMetadata {
                name: "required_field".to_string(),
                field_type: FieldType::String,
                nullability: Nullability::Required,
                metadata: HashMap::new(),
            },
        ]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        // Adding required field is breaking
        assert!(migration.is_breaking);
        assert!(!SchemaEvolution::is_backward_compatible(&migration));
    }
}
