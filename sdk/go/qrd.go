// Package qrd provides Go bindings for the QRD columnar binary format.
//
// This package uses CGO to interface with the Rust QRD FFI layer.
package qrd

/*
#cgo LDFLAGS: -L../../target/release -lqrd_ffi
#include <stdlib.h>
#include "qrd.h"
*/
import "C"

import (
	"errors"
	"fmt"
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
	FieldTypeString
	FieldTypeBlob
	FieldTypeUuid
	FieldTypeDecimal
)

// Nullability represents whether a field can be null
type Nullability int

const (
	NullabilityRequired Nullability = iota
	NullabilityOptional
)

// getLastError retrieves the last error from FFI
func getLastError() string {
	cErr := C.qrd_last_error()
	if cErr != nil {
		return C.GoString(cErr)
	}
	return "unknown error"
}

// SchemaBuilder builds QRD schemas
type SchemaBuilder struct {
	ptr *C.QrdSchemaBuilder
}

// NewSchemaBuilder creates a new schema builder
func NewSchemaBuilder() *SchemaBuilder {
	return &SchemaBuilder{ptr: C.qrd_schema_builder_new()}
}

// Free releases the schema builder resources
func (sb *SchemaBuilder) Free() {
	if sb.ptr != nil {
		C.qrd_schema_builder_free(sb.ptr)
		sb.ptr = nil
	}
}

// AddField adds a field to the schema
func (sb *SchemaBuilder) AddField(name string, fieldType int, nullable bool) error {
	if sb.ptr == nil {
		return errors.New("schema builder has been freed")
	}
	
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	nullability := C.int(0) // Required
	if nullable {
		nullability = C.int(1) // Optional
	}

	result := C.qrd_schema_builder_add_field(
		sb.ptr,
		cName,
		C.int(fieldType),
		nullability,
	)
	if result != 0 {
		return fmt.Errorf("failed to add field: %s", getLastError())
	}
	return nil
}

// Build creates the schema
func (sb *SchemaBuilder) Build() (*Schema, error) {
	if sb.ptr == nil {
		return nil, errors.New("schema builder has been freed")
	}
	
	ptr := C.qrd_schema_builder_build(sb.ptr)
	if ptr == nil {
		return nil, fmt.Errorf("failed to build schema: %s", getLastError())
	}
	sb.ptr = nil // Builder is consumed
	return &Schema{ptr: ptr}, nil
}

// Schema represents a QRD schema
type Schema struct {
	ptr *C.QrdSchema
}

// Free releases the schema resources
func (s *Schema) Free() {
	if s.ptr != nil {
		C.qrd_schema_free(s.ptr)
		s.ptr = nil
	}
}

// FieldCount returns the number of fields in the schema
func (s *Schema) FieldCount() uint32 {
	if s.ptr == nil {
		return 0
	}
	return uint32(C.qrd_schema_field_count(s.ptr))
}

// Writer writes QRD files
type Writer struct {
	ptr *C.QrdWriter
}

// NewWriter creates a new writer
func NewWriter(path string, schema *Schema) (*Writer, error) {
	if schema.ptr == nil {
		return nil, errors.New("schema is invalid")
	}
	
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	ptr := C.qrd_writer_new(cPath, schema.ptr)
	if ptr == nil {
		return nil, fmt.Errorf("failed to create writer: %s", getLastError())
	}
	return &Writer{ptr: ptr}, nil
}

// Free releases the writer resources
func (w *Writer) Free() {
	if w.ptr != nil {
		C.qrd_writer_free(w.ptr)
		w.ptr = nil
	}
}

// WriteRow writes a row to the file
func (w *Writer) WriteRow(columns [][]byte) error {
	if w.ptr == nil {
		return errors.New("writer has been freed or finished")
	}
	
	// Create pointers array
	dataPtrs := make([]*C.uint8_t, len(columns))
	dataLens := make([]C.uint32_t, len(columns))
	
	for i, col := range columns {
		if len(col) > 0 {
			dataPtrs[i] = (*C.uint8_t)(unsafe.Pointer(&col[0]))
		}
		dataLens[i] = C.uint32_t(len(col))
	}
	
	result := C.qrd_writer_write_row(
		w.ptr,
		C.uint32_t(len(columns)),
		(**C.uint8_t)(unsafe.Pointer(&dataPtrs[0])),
		(*C.uint32_t)(unsafe.Pointer(&dataLens[0])),
	)
	
	if result != 0 {
		return fmt.Errorf("failed to write row: %s", getLastError())
	}
	return nil
}

// Finish closes the writer and finalizes the file
func (w *Writer) Finish() error {
	if w.ptr == nil {
		return errors.New("writer has been freed or finished")
	}
	
	result := C.qrd_writer_finish(w.ptr)
	if result != 0 {
		return fmt.Errorf("failed to finish writing: %s", getLastError())
	}
	w.ptr = nil
	return nil
}

// Reader reads QRD files
type Reader struct {
	ptr *C.QrdReader
}

// NewReader opens a QRD file for reading
func NewReader(path string) (*Reader, error) {
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	ptr := C.qrd_reader_new(cPath)
	if ptr == nil {
		return nil, fmt.Errorf("failed to create reader: %s", getLastError())
	}
	return &Reader{ptr: ptr}, nil
}

// Free releases the reader resources
func (r *Reader) Free() {
	if r.ptr != nil {
		C.qrd_reader_free(r.ptr)
		r.ptr = nil
	}
}

// RowCount returns the number of rows in the file
func (r *Reader) RowCount() uint32 {
	if r.ptr == nil {
		return 0
	}
	return uint32(C.qrd_reader_row_count(r.ptr))
}