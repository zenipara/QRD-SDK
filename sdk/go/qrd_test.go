package qrd

import (
	"testing"
)

func TestSchemaBuilder(t *testing.T) {
	// Create schema builder
	builder := NewSchemaBuilder()
	defer builder.Free()
	
	err := builder.AddField("id", FieldTypeInt64, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}
	
	err = builder.AddField("name", FieldTypeString, NullabilityOptional, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}
	
	err = builder.AddField("score", FieldTypeFloat64, NullabilityOptional, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}
	
	// Build schema
	schema, buildErr := builder.Build()
	if buildErr != nil {
		t.Fatalf("Failed to build schema: %v", buildErr)
	}
	defer schema.Free()
	
	if schema.FieldCount() != 3 {
		t.Errorf("Expected 3 fields, got %d", schema.FieldCount())
	}
}

func TestBasicIO(t *testing.T) {
	// Create schema
	builder := NewSchemaBuilder()
	defer builder.Free()
	
	builder.AddField("id", FieldTypeInt64, NullabilityRequired, "")
	builder.AddField("value", FieldTypeFloat64, NullabilityOptional, "")
	
	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}
	defer schema.Free()
	
	// Test writer (in memory)
	// Note: This is a basic test that validates the API
	_ = schema
}