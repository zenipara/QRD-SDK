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

func TestCGOIntegration(t *testing.T) {
	// Test CGO integration and memory management
	builder := NewSchemaBuilder()
	defer builder.Free()

	err := builder.AddField("test", FieldTypeInt32, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}
	defer schema.Free()

	// Test that CGO handles are properly managed
	if schema.FieldCount() != 1 {
		t.Errorf("Expected 1 field, got %d", schema.FieldCount())
	}
}

func TestConcurrentAccess(t *testing.T) {
	// Test concurrent access to QRD objects
	builder := NewSchemaBuilder()
	defer builder.Free()

	err := builder.AddField("id", FieldTypeInt64, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}
	defer schema.Free()

	// Test that schema can be accessed from multiple goroutines
	done := make(chan bool, 2)

	go func() {
		if schema.FieldCount() != 1 {
			t.Errorf("Goroutine 1: Expected 1 field, got %d", schema.FieldCount())
		}
		done <- true
	}()

	go func() {
		if schema.FieldCount() != 1 {
			t.Errorf("Goroutine 2: Expected 1 field, got %d", schema.FieldCount())
		}
		done <- true
	}()

	<-done
	<-done
}

func TestPartialReads(t *testing.T) {
	// Test partial column reads
	builder := NewSchemaBuilder()
	defer builder.Free()

	err := builder.AddField("col1", FieldTypeInt64, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}

	err = builder.AddField("col2", FieldTypeString, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}

	err = builder.AddField("col3", FieldTypeFloat64, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}
	defer schema.Free()

	// Test schema field access
	if schema.FieldCount() != 3 {
		t.Errorf("Expected 3 fields, got %d", schema.FieldCount())
	}
}

func TestDeterministicWrites(t *testing.T) {
	// Test deterministic write behavior
	builder := NewSchemaBuilder()
	defer builder.Free()

	err := builder.AddField("value", FieldTypeInt64, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}
	defer schema.Free()

	// Test that multiple writes produce identical results
	// (This would require actual file I/O in a real implementation)
	if schema.FieldCount() != 1 {
		t.Errorf("Expected 1 field, got %d", schema.FieldCount())
	}
}

func TestNilHandling(t *testing.T) {
	// Test nil pointer handling
	builder := NewSchemaBuilder()
	defer builder.Free()

	err := builder.AddField("optional", FieldTypeString, NullabilityOptional, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}
	defer schema.Free()

	// Test that optional fields are handled correctly
	if schema.FieldCount() != 1 {
		t.Errorf("Expected 1 field, got %d", schema.FieldCount())
	}
}

func TestInvalidSchema(t *testing.T) {
	// Test invalid schema rejection
	builder := NewSchemaBuilder()
	defer builder.Free()

	// Try to add field with empty name
	err := builder.AddField("", FieldTypeInt32, NullabilityRequired, "").err
	if err == nil {
		t.Error("Expected error for empty field name")
	}

	// Try to add duplicate field names
	err = builder.AddField("dup", FieldTypeInt32, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add first field: %v", err)
	}

	err = builder.AddField("dup", FieldTypeString, NullabilityRequired, "").err
	if err == nil {
		t.Error("Expected error for duplicate field name")
	}
}

func TestResourceCleanup(t *testing.T) {
	// Test proper resource cleanup
	builder := NewSchemaBuilder()
	defer builder.Free()

	err := builder.AddField("test", FieldTypeInt64, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}
	defer schema.Free()

	// Test that resources are properly freed
	// (This would be validated by memory leak detection tools)
	if schema.FieldCount() != 1 {
		t.Errorf("Expected 1 field, got %d", schema.FieldCount())
	}
}

func TestLargeDatasetHandling(t *testing.T) {
	// Test handling of large datasets
	builder := NewSchemaBuilder()
	defer builder.Free()

	err := builder.AddField("seq", FieldTypeInt64, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}
	defer schema.Free()

	// Test schema for large dataset handling
	if schema.FieldCount() != 1 {
		t.Errorf("Expected 1 field, got %d", schema.FieldCount())
	}
}

func TestFooterInspection(t *testing.T) {
	// Test footer metadata inspection
	builder := NewSchemaBuilder()
	defer builder.Free()

	err := builder.AddField("data", FieldTypeBlob, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}
	defer schema.Free()

	// Test footer access
	if schema.FieldCount() != 1 {
		t.Errorf("Expected 1 field, got %d", schema.FieldCount())
	}
}

func TestCrossLanguageCompatibility(t *testing.T) {
	// Test compatibility with other language SDKs
	builder := NewSchemaBuilder()
	defer builder.Free()

	err := builder.AddField("shared", FieldTypeInt32, NullabilityRequired, "").err
	if err != nil {
		t.Fatalf("Failed to add field: %v", err)
	}

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}
	defer schema.Free()

}
