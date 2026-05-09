package qrd

import (
	"testing"
)

func TestSchemaBuilder(t *testing.T) {
	// Create schema builder
	builder := NewSchemaBuilder()
	defer builder.Free()
	
	err := builder.AddField("id", int(FieldTypeInt64), false)
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}
	
	err = builder.AddField("name", int(FieldTypeString), true)
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}
	
	err = builder.AddField("score", int(FieldTypeFloat64), true)
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}
	
	// Build schema
	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
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
	
	builder.AddField("id", int(FieldTypeInt64), false)
	builder.AddField("value", int(FieldTypeFloat64), true)
	
	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}
	defer schema.Free()
	
	// Test writer (in memory)
	// Note: This is a basic test that validates the API
	_ = schema
}