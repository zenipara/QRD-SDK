// Package qrd provides Go bindings for the QRD columnar binary format.
//
// This package uses CGO to interface with the Rust QRD core library.
package qrd

/*
#cgo CFLAGS: -I../../core/qrd-ffi
#cgo LDFLAGS: -L../../target/release -lqrd_ffi
#include "qrd.h"
*/
import "C"

import (
	"errors"
	"runtime"
	"unsafe"
)

// FieldType represents the logical type of a schema field
type FieldType int

const (
	FieldTypeBoolean FieldType = iota
	FieldTypeInt8
	FieldTypeInt16
	FieldTypeInt32
	FieldTypeInt64
	FieldTypeUint8
	FieldTypeUint16
	FieldTypeUint32
	FieldTypeUint64
	FieldTypeFloat32
	FieldTypeFloat64
	FieldTypeTimestamp
	FieldTypeDate
	FieldTypeTime
	FieldTypeDuration
	FieldTypeString
	FieldTypeEnum
	FieldTypeUUID
	FieldTypeBlob
	FieldTypeDecimal
)

// Nullability represents whether a field can be null
type Nullability int

const (
	NullabilityRequired Nullability = iota
	NullabilityOptional
	NullabilityRepeated
)

// Field represents a schema field definition
type Field struct {
	Name        string
	Type        FieldType
	Nullability Nullability
	Metadata    string
}

// Schema represents a QRD schema
type Schema struct {
	handle unsafe.Pointer
	id     uint64
}

// NewSchema creates a new schema builder
func NewSchema() *SchemaBuilder {
	return &SchemaBuilder{
		fields: make([]Field, 0),
	}
}

// SchemaBuilder builds QRD schemas
type SchemaBuilder struct {
	fields []Field
}

// AddField adds a field to the schema
func (sb *SchemaBuilder) AddField(name string, fieldType FieldType, nullability Nullability, metadata string) *SchemaBuilder {
	sb.fields = append(sb.fields, Field{
		Name:        name,
		Type:        fieldType,
		Nullability: nullability,
		Metadata:    metadata,
	})
	return sb
}

// Build creates the schema
func (sb *SchemaBuilder) Build() (*Schema, error) {
	// Create C schema
	cSchema := C.qrd_schema_new()
	if cSchema == nil {
		return nil, errors.New("failed to create schema")
	}

	// Add fields
	for _, field := range sb.fields {
		cName := C.CString(field.Name)
		defer C.free(unsafe.Pointer(cName))

		var cMetadata *C.char
		if field.Metadata != "" {
			cMetadata = C.CString(field.Metadata)
			defer C.free(unsafe.Pointer(cMetadata))
		}

		result := C.qrd_schema_add_field(cSchema, cName, C.int(field.Type), C.int(field.Nullability), cMetadata)
		if result != 0 {
			C.qrd_schema_free(cSchema)
			return nil, errors.New("failed to add field to schema")
		}
	}

	schema := &Schema{
		handle: cSchema,
		id:     uint64(C.qrd_schema_id(cSchema)),
	}

	// Set finalizer to free C resources
	runtime.SetFinalizer(schema, func(s *Schema) {
		C.qrd_schema_free(s.handle)
	})

	return schema, nil
}

// ID returns the schema ID
func (s *Schema) ID() uint64 {
	return s.id
}

// FieldCount returns the number of fields in the schema
func (s *Schema) FieldCount() int {
	return int(C.qrd_schema_field_count(s.handle))
}

// FileWriter writes QRD files
type FileWriter struct {
	handle unsafe.Pointer
}

// NewFileWriter creates a new file writer
func NewFileWriter(schema *Schema) (*FileWriter, error) {
	handle := C.qrd_writer_new(schema.handle)
	if handle == nil {
		return nil, errors.New("failed to create writer")
	}

	writer := &FileWriter{handle: handle}
	runtime.SetFinalizer(writer, func(w *FileWriter) {
		C.qrd_writer_free(w.handle)
	})

	return writer, nil
}

// WriteRow writes a row of data
func (w *FileWriter) WriteRow(row []interface{}) error {
	// Convert Go row to C representation
	// This is a simplified implementation - in practice you'd need proper type conversion
	cRow := C.qrd_row_new()
	if cRow == nil {
		return errors.New("failed to create row")
	}
	defer C.qrd_row_free(cRow)

	// Add values to row (simplified)
	for _, value := range row {
		switch v := value.(type) {
		case int64:
			C.qrd_row_add_int64(cRow, C.longlong(v))
		case float64:
			C.qrd_row_add_float64(cRow, C.double(v))
		case string:
			cStr := C.CString(v)
			C.qrd_row_add_string(cRow, cStr)
			C.free(unsafe.Pointer(cStr))
		// Add other types...
		}
	}

	result := C.qrd_writer_write_row(w.handle, cRow)
	if result != 0 {
		return errors.New("failed to write row")
	}

	return nil
}

// Finish finishes writing and returns the data
func (w *FileWriter) Finish() ([]byte, error) {
	var cData *C.uint8_t
	var cSize C.size_t

	result := C.qrd_writer_finish(w.handle, &cData, &cSize)
	if result != 0 {
		return nil, errors.New("failed to finish writing")
	}

	// Copy C data to Go slice
	data := make([]byte, int(cSize))
	C.memcpy(unsafe.Pointer(&data[0]), unsafe.Pointer(cData), cSize)

	// Free C data
	C.free(unsafe.Pointer(cData))

	return data, nil
}

// FileReader reads QRD files
type FileReader struct {
	handle unsafe.Pointer
	schema *Schema
}

// NewFileReader creates a new file reader
func NewFileReader(data []byte) (*FileReader, error) {
	cData := (*C.uint8_t)(unsafe.Pointer(&data[0]))
	cSize := C.size_t(len(data))

	handle := C.qrd_reader_new(cData, cSize)
	if handle == nil {
		return nil, errors.New("failed to create reader")
	}

	// Get schema
	cSchema := C.qrd_reader_schema(handle)
	if cSchema == nil {
		C.qrd_reader_free(handle)
		return nil, errors.New("failed to get schema")
	}

	schema := &Schema{
		handle: cSchema,
		id:     uint64(C.qrd_schema_id(cSchema)),
	}

	reader := &FileReader{
		handle: handle,
		schema: schema,
	}

	runtime.SetFinalizer(reader, func(r *FileReader) {
		C.qrd_reader_free(r.handle)
	})

	return reader, nil
}

// Schema returns the file schema
func (r *FileReader) Schema() *Schema {
	return r.schema
}

// RowCount returns the number of rows
func (r *FileReader) RowCount() uint64 {
	return uint64(C.qrd_reader_row_count(r.handle))
}

// ReadRow reads the next row
func (r *FileReader) ReadRow() ([]interface{}, error) {
	cRow := C.qrd_reader_read_row(r.handle)
	if cRow == nil {
		return nil, nil // EOF
	}
	defer C.qrd_row_free(cRow)

	// Convert C row to Go slice
	// This is simplified - in practice you'd need proper type conversion
	row := make([]interface{}, 0)

	fieldCount := int(C.qrd_row_field_count(cRow))
	for i := 0; i < fieldCount; i++ {
		// Get field value based on type
		// This would need to be implemented based on the actual C API
		row = append(row, nil) // Placeholder
	}

	return row, nil
}

// ReadAllRows reads all rows at once
func (r *FileReader) ReadAllRows() ([][]interface{}, error) {
	var rows [][]interface{}
	for {
		row, err := r.ReadRow()
		if err != nil {
			return nil, err
		}
		if row == nil {
			break
		}
		rows = append(rows, row)
	}
	return rows, nil
}