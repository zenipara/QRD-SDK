//! Integration tests for schema evolution
//!
//! Tests field rename, type change, and migration scenarios

#[cfg(test)]
mod schema_evolution_tests {
    use qrd_core::schema::{
        FieldMetadata, FieldType, Nullability, Schema, SchemaEvolution, SchemaMigration,
        TypeCompatibility,
    };
    use std::collections::HashMap;

    fn create_field(name: &str, field_type: FieldType, nullability: Nullability) -> FieldMetadata {
        FieldMetadata {
            name: name.to_string(),
            field_type,
            nullability,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_simple_field_rename() {
        // Scenario: Rename 'user_id' to 'id' (type stays the same)
        // NOTE: Without explicit rename mapping, this appears as field removal + addition
        let old_schema = Schema::new(vec![
            create_field("user_id", FieldType::Int64, Nullability::Required),
            create_field("name", FieldType::String, Nullability::Optional),
        ]);

        let new_schema = Schema::new(vec![
            create_field("id", FieldType::Int64, Nullability::Required),
            create_field("name", FieldType::String, Nullability::Optional),
        ]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        // Without explicit rename mapping, 'user_id' is removed and 'id' is added
        // This is the limitation of name-based matching
        assert_eq!(migration.removed_field_indices.len(), 1); // user_id removed
        assert_eq!(migration.new_fields.len(), 1); // id added

        // But 'name' field should be mapped since the name matches
        assert!(migration.field_mappings[1].is_some());
        let mapping = migration.field_mappings[1].as_ref().unwrap();
        assert_eq!(mapping.old_name, "name");
        assert_eq!(mapping.new_name, "name");
    }

    #[test]
    fn test_field_type_widening() {
        // Scenario: Change Int32 to Int64 (widening - safe)
        let old_schema = Schema::new(vec![create_field("count", FieldType::Int32, Nullability::Required)]);

        let new_schema = Schema::new(vec![create_field("count", FieldType::Int64, Nullability::Required)]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        assert!(migration.field_mappings[0].is_some());
        let mapping = migration.field_mappings[0].as_ref().unwrap();
        assert_eq!(mapping.compatibility, TypeCompatibility::Widening);
        assert!(!migration.is_breaking);
    }

    #[test]
    fn test_field_type_narrowing() {
        // Scenario: Change Int64 to Int32 (narrowing - potentially lossy)
        let old_schema = Schema::new(vec![create_field("count", FieldType::Int64, Nullability::Required)]);

        let new_schema = Schema::new(vec![create_field("count", FieldType::Int32, Nullability::Required)]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        assert!(migration.field_mappings[0].is_some());
        let mapping = migration.field_mappings[0].as_ref().unwrap();
        assert_eq!(mapping.compatibility, TypeCompatibility::Narrowing);
        // Narrowing alone is not breaking
        assert!(!migration.is_breaking);
    }

    #[test]
    fn test_incompatible_type_change() {
        // Scenario: Change Int64 to Boolean (incompatible)
        let old_schema = Schema::new(vec![create_field("value", FieldType::Int64, Nullability::Required)]);

        let new_schema = Schema::new(vec![create_field("value", FieldType::Boolean, Nullability::Required)]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        assert!(migration.field_mappings[0].is_some());
        let mapping = migration.field_mappings[0].as_ref().unwrap();
        assert_eq!(mapping.compatibility, TypeCompatibility::Incompatible);
        assert!(migration.is_breaking);
    }

    #[test]
    fn test_field_addition_optional() {
        // Scenario: Add optional field 'created_at'
        let old_schema = Schema::new(vec![create_field("id", FieldType::Int64, Nullability::Required)]);

        let new_schema = Schema::new(vec![
            create_field("id", FieldType::Int64, Nullability::Required),
            create_field("created_at", FieldType::Timestamp, Nullability::Optional),
        ]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        // Should have 1 new field
        assert_eq!(migration.new_fields.len(), 1);
        assert_eq!(migration.new_fields[0].name, "created_at");

        // Should NOT be breaking (optional field)
        assert!(!migration.is_breaking);

        // Should be backward compatible (new reader can read old data)
        assert!(SchemaEvolution::is_backward_compatible(&migration));
    }

    #[test]
    fn test_field_addition_required_breaking() {
        // Scenario: Add required field 'email'
        let old_schema = Schema::new(vec![create_field("id", FieldType::Int64, Nullability::Required)]);

        let new_schema = Schema::new(vec![
            create_field("id", FieldType::Int64, Nullability::Required),
            create_field("email", FieldType::String, Nullability::Required),
        ]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        // Should have 1 new required field
        assert_eq!(migration.new_fields.len(), 1);
        assert_eq!(migration.new_fields[0].nullability, Nullability::Required);

        // SHOULD be breaking (required field added)
        assert!(migration.is_breaking);

        // NOT backward compatible
        assert!(!SchemaEvolution::is_backward_compatible(&migration));
    }

    #[test]
    fn test_field_removal() {
        // Scenario: Remove optional field 'deprecated_field'
        let old_schema = Schema::new(vec![
            create_field("id", FieldType::Int64, Nullability::Required),
            create_field("deprecated_field", FieldType::String, Nullability::Optional),
        ]);

        let new_schema = Schema::new(vec![create_field("id", FieldType::Int64, Nullability::Required)]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        // Should have 1 removed field
        assert_eq!(migration.removed_field_indices.len(), 1);
        assert_eq!(migration.removed_field_indices[0], 1); // Index of 'deprecated_field'

        // NOT breaking (optional field removed)
        assert!(!migration.is_breaking);

        // Still forward compatible (old reader can read new data)
        assert!(SchemaEvolution::is_forward_compatible(&migration));
    }

    #[test]
    fn test_field_removal_required_breaking() {
        // Scenario: Remove required field 'user_id' - BREAKING
        let old_schema = Schema::new(vec![
            create_field("user_id", FieldType::Int64, Nullability::Required),
            create_field("name", FieldType::String, Nullability::Optional),
        ]);

        let new_schema = Schema::new(vec![create_field("name", FieldType::String, Nullability::Optional)]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        // SHOULD be breaking (required field removed)
        assert!(migration.is_breaking);
    }

    #[test]
    fn test_rename_and_type_change() {
        // Scenario: Rename field AND change type
        let old_schema = Schema::new(vec![create_field("user_count", FieldType::Int32, Nullability::Required)]);

        let new_schema = Schema::new(vec![create_field("total_users", FieldType::Int64, Nullability::Required)]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        // Field will be detected as removed (name changed completely)
        // This is a limitation - needs explicit rename mapping
        assert_eq!(migration.removed_field_indices.len(), 1);
        assert_eq!(migration.new_fields.len(), 1);
    }

    #[test]
    fn test_complex_migration_scenario() {
        // Scenario: Real-world migration
        // Old: [user_id (Int64), first_name (String), last_name (String)]
        // New: [id (Int64), full_name (String), email (String, Optional), created_at (Timestamp, Optional)]
        // NOTE: Without explicit field mapping, renamed fields appear as removals + additions
        
        let old_schema = Schema::new(vec![
            create_field("user_id", FieldType::Int64, Nullability::Required),
            create_field("first_name", FieldType::String, Nullability::Optional),
            create_field("last_name", FieldType::String, Nullability::Optional),
        ]);

        let new_schema = Schema::new(vec![
            create_field("id", FieldType::Int64, Nullability::Required),
            create_field("full_name", FieldType::String, Nullability::Optional),
            create_field("email", FieldType::String, Nullability::Optional),
            create_field("created_at", FieldType::Timestamp, Nullability::Optional),
        ]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        // All 3 old fields are removed (name-based matching fails due to renames)
        assert_eq!(migration.removed_field_indices.len(), 3);

        // All 4 new fields are added
        assert_eq!(migration.new_fields.len(), 4);

        // IS breaking because required field (user_id) is removed
        assert!(migration.is_breaking);
    }

    #[test]
    fn test_type_compatibility_numeric() {
        // Test numeric type compatibility
        assert_eq!(
            SchemaEvolution::check_type_compatibility(FieldType::Int8, FieldType::Int16),
            TypeCompatibility::Widening
        );
        assert_eq!(
            SchemaEvolution::check_type_compatibility(FieldType::Int16, FieldType::Int8),
            TypeCompatibility::Narrowing
        );
        assert_eq!(
            SchemaEvolution::check_type_compatibility(FieldType::Float32, FieldType::Float64),
            TypeCompatibility::Widening
        );
    }

    #[test]
    fn test_type_compatibility_string_conversions() {
        // UUID <-> String conversions
        assert_eq!(
            SchemaEvolution::check_type_compatibility(FieldType::Uuid, FieldType::String),
            TypeCompatibility::Widening
        );
        assert_eq!(
            SchemaEvolution::check_type_compatibility(FieldType::String, FieldType::Uuid),
            TypeCompatibility::Narrowing
        );

        // Blob <-> String conversions
        assert_eq!(
            SchemaEvolution::check_type_compatibility(FieldType::String, FieldType::Blob),
            TypeCompatibility::Widening
        );
        assert_eq!(
            SchemaEvolution::check_type_compatibility(FieldType::Blob, FieldType::String),
            TypeCompatibility::Narrowing
        );
    }

    #[test]
    fn test_type_compatibility_incompatible() {
        // Incompatible conversions
        assert_eq!(
            SchemaEvolution::check_type_compatibility(FieldType::Int64, FieldType::Boolean),
            TypeCompatibility::Incompatible
        );
        assert_eq!(
            SchemaEvolution::check_type_compatibility(FieldType::String, FieldType::Int64),
            TypeCompatibility::Incompatible
        );
        assert_eq!(
            SchemaEvolution::check_type_compatibility(FieldType::Timestamp, FieldType::Blob),
            TypeCompatibility::Incompatible
        );
    }

    #[test]
    fn test_nullability_change() {
        // Scenario: Change from Required to Optional (should work)
        let old_schema = Schema::new(vec![create_field("value", FieldType::Int64, Nullability::Required)]);

        let new_schema = Schema::new(vec![create_field("value", FieldType::Int64, Nullability::Optional)]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        assert!(migration.field_mappings[0].is_some());
        let mapping = migration.field_mappings[0].as_ref().unwrap();
        assert!(mapping.nullability_changed);
        assert!(!migration.is_breaking);
    }

    #[test]
    fn test_nullability_change_optional_to_required() {
        // Scenario: Change from Optional to Required (could be breaking)
        let old_schema = Schema::new(vec![create_field("value", FieldType::Int64, Nullability::Optional)]);

        let new_schema = Schema::new(vec![create_field("value", FieldType::Int64, Nullability::Required)]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        assert!(migration.field_mappings[0].is_some());
        let mapping = migration.field_mappings[0].as_ref().unwrap();
        assert!(mapping.nullability_changed);
        // This is potentially breaking for old readers
        assert!(!migration.is_breaking); // But auto-detection won't catch it
    }

    #[test]
    fn test_forward_backward_compatibility_assessment() {
        let old_schema = Schema::new(vec![
            create_field("id", FieldType::Int64, Nullability::Required),
            create_field("name", FieldType::String, Nullability::Optional),
        ]);

        let new_schema = Schema::new(vec![
            create_field("id", FieldType::Int64, Nullability::Required),
            create_field("name", FieldType::String, Nullability::Optional),
            create_field("email", FieldType::String, Nullability::Optional),
        ]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        // Forward compatible: old reader can read new data
        assert!(SchemaEvolution::is_forward_compatible(&migration));
        // Backward compatible: new reader can read old data
        assert!(SchemaEvolution::is_backward_compatible(&migration));
    }

    #[test]
    fn test_type_conversions_availability() {
        let conversions = SchemaEvolution::get_type_conversions();
        
        // Should have conversions
        assert!(!conversions.is_empty());
        
        // Check some expected conversions
        let safe_conversions: Vec<_> = conversions.iter().filter(|c| c.is_safe).collect();
        let unsafe_conversions: Vec<_> = conversions.iter().filter(|c| !c.is_safe).collect();
        
        assert!(!safe_conversions.is_empty(), "Should have safe conversions");
        assert!(!unsafe_conversions.is_empty(), "Should have unsafe conversions");
        
        // UUID to String should be safe
        assert!(conversions
            .iter()
            .any(|c| c.name == "uuid_to_string" && c.is_safe));
        
        // String to UUID should be unsafe
        assert!(conversions
            .iter()
            .any(|c| c.name == "string_to_uuid" && !c.is_safe));
    }

    #[test]
    fn test_repeated_field_handling() {
        // Scenario: Change field from Optional to Repeated
        let old_schema = Schema::new(vec![
            create_field("id", FieldType::Int64, Nullability::Required),
            create_field("tags", FieldType::String, Nullability::Optional),
        ]);

        let new_schema = Schema::new(vec![
            create_field("id", FieldType::Int64, Nullability::Required),
            create_field("tags", FieldType::String, Nullability::Repeated),
        ]);

        let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema).unwrap();

        assert!(migration.field_mappings[1].is_some());
        let mapping = migration.field_mappings[1].as_ref().unwrap();
        assert!(mapping.nullability_changed);
    }
}
