package qrd

import (
	"testing"
)

func TestSchemaBuilder(t *testing.T) {
	schema, err := NewSchema().
		AddField("id", FieldTypeInt64, NullabilityRequired, "").
		AddField("name", FieldTypeString, NullabilityOptional, "").
		AddField("score", FieldTypeFloat64, NullabilityOptional, "").
		Build()

	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}

	if schema.ID() == 0 {
		t.Error("Schema ID should not be zero")
	}

	if schema.FieldCount() != 3 {
		t.Errorf("Expected 3 fields, got %d", schema.FieldCount())
	}
}

func TestFileWriterReader(t *testing.T) {
	// Create schema
	schema, err := NewSchema().
		AddField("id", FieldTypeInt64, NullabilityRequired, "").
		AddField("value", FieldTypeFloat64, NullabilityOptional, "").
		Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}

	// Create writer
	writer, err := NewFileWriter(schema)
	if err != nil {
		t.Fatalf("Failed to create writer: %v", err)
	}

	// Write test data
	testData := []struct {
		id    int64
		value float64
	}{
		{1, 3.14},
		{2, 2.71},
		{3, 1.41},
	}

	for _, data := range testData {
		err = writer.WriteRow([]interface{}{data.id, data.value})
		if err != nil {
			t.Fatalf("Failed to write row: %v", err)
		}
	}

	// Finish writing
	buffer, err := writer.Finish()
	if err != nil {
		t.Fatalf("Failed to finish writing: %v", err)
	}

	if len(buffer) == 0 {
		t.Error("Buffer should not be empty")
	}

	// Create reader
	reader, err := NewFileReader(buffer)
	if err != nil {
		t.Fatalf("Failed to create reader: %v", err)
	}

	if reader.RowCount() != uint64(len(testData)) {
		t.Errorf("Expected %d rows, got %d", len(testData), reader.RowCount())
	}

	// Read all rows
	rows, err := reader.ReadAllRows()
	if err != nil {
		t.Fatalf("Failed to read rows: %v", err)
	}

	if len(rows) != len(testData) {
		t.Errorf("Expected %d rows, got %d", len(testData), len(rows))
	}
}

func TestStringFields(t *testing.T) {
	schema, err := NewSchema().
		AddField("name", FieldTypeString, NullabilityRequired, "").
		AddField("description", FieldTypeString, NullabilityOptional, "").
		Build()
	if err != nil {
		t.Fatalf("Failed to build schema: %v", err)
	}

	writer, err := NewFileWriter(schema)
	if err != nil {
		t.Fatalf("Failed to create writer: %v", err)
	}

	// Write string data
	err = writer.WriteRow([]interface{}{"Alice", "Software Engineer"})
	if err != nil {
		t.Fatalf("Failed to write string row: %v", err)
	}

	err = writer.WriteRow([]interface{}{"Bob", nil}) // Test optional field
	if err != nil {
		t.Fatalf("Failed to write string row with nil: %v", err)
	}

	buffer, err := writer.Finish()
	if err != nil {
		t.Fatalf("Failed to finish writing: %v", err)
	}

	reader, err := NewFileReader(buffer)
	if err != nil {
		t.Fatalf("Failed to create reader: %v", err)
	}

	if reader.RowCount() != 2 {
		t.Errorf("Expected 2 rows, got %d", reader.RowCount())
	}
}