// Package qrd provides Go bindings for the QRD columnar binary format.
//
// This package uses CGO to interface with the Rust QRD FFI layer.
package qrd

/*
#cgo LDFLAGS: -L../../target/release -lqrd_ffi
#include "../../core/qrd-ffi/src/lib.rs"
*/
import "C"

import (
	"errors"
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

// SchemaBuilder builds QRD schemas
type SchemaBuilder struct {
	ptr unsafe.Pointer
}

// NewSchemaBuilder creates a new schema builder
func NewSchemaBuilder() *SchemaBuilder {
	return &SchemaBuilder{ptr: unsafe.Pointer(C.qrd_schema_builder_new())}
}

// Free releases the schema builder resources
func (sb *SchemaBuilder) Free() {
	if sb.ptr != nil {
		C.qrd_schema_builder_free((*C.QrdSchemaBuilder)(sb.ptr))
		sb.ptr = nil
	}
}

// AddField adds a field to the schema
func (sb *SchemaBuilder) AddField(name string, fieldType int, nullability int) error {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	result := C.qrd_schema_builder_add_field(
		(*C.QrdSchemaBuilder)(sb.ptr),
		cName,
		C.int(fieldType),
		C.int(nullability),
	)
	if result != 0 {
		cErr := C.qrd_last_error()
		if cErr != nil {
			return errors.New(C.GoString(cErr))
		}
		return errors.New("failed to add field to schema")
	}
	return nil
}

// Build creates the schema
func (sb *SchemaBuilder) Build() (*Schema, error) {
	ptr := C.qrd_schema_builder_build((*C.QrdSchemaBuilder)(sb.ptr))
	if ptr == nil {
		cErr := C.qrd_last_error()
		if cErr != nil {
			return nil, errors.New(C.GoString(cErr))
		}
		return nil, errors.New("failed to build schema")
	}
	return &Schema{ptr: unsafe.Pointer(ptr)}, nil
}

// Schema represents a QRD schema
type Schema struct {
	ptr unsafe.Pointer
}

// Free releases the schema resources
func (s *Schema) Free() {
	if s.ptr != nil {
		C.qrd_schema_free((*C.QrdSchema)(s.ptr))
		s.ptr = nil
	}
}

// FileWriter writes QRD files
type FileWriter struct {
	ptr unsafe.Pointer
}

// NewFileWriter creates a new file writer
func NewFileWriter(path string, schema *Schema) (*FileWriter, error) {
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	ptr := C.qrd_writer_new(cPath, (*C.QrdSchema)(schema.ptr))
	if ptr == nil {
		cErr := C.qrd_last_error()
		if cErr != nil {
			return nil, errors.New(C.GoString(cErr))
		}
		return nil, errors.New("failed to create writer")
	}
	return &FileWriter{ptr: unsafe.Pointer(ptr)}, nil
}

// Free releases the writer resources  
func (w *FileWriter) Free() {
	if w.ptr != nil {
		C.qrd_writer_free((*C.QrdWriter)(w.ptr))
		w.ptr = nil
	}
}

// WriteRow writes a row of data. columnData is array of byte slices.
func (w *FileWriter) WriteRow(columns [][]byte) error {
	// Convert to C arrays
	dataPtrs := make([]*C.uint8_t, len(columns))
	dataLens := make([]C.uint, len(columns))

	for i, col := range columns {
		if len(col) > 0 {
			dataPtrs[i] = (*C.uint8_t)(unsafe.Pointer(&col[0]))
		}
		dataLens[i] = C.uint(len(col))
	}

	result := C.qrd_writer_write_row(
		(*C.QrdWriter)(w.ptr),
		C.uint(len(columns)),
		&dataPtrs[0],
		&dataLens[0],
	)
	if result != 0 {
		cErr := C.qrd_last_error()
		if cErr != nil {
			return errors.New(C.GoString(cErr))
		}
		return errors.New("failed to write row")
	}
	return nil
}

// Finish finishes writing the file
func (w *FileWriter) Finish() error {
	result := C.qrd_writer_finish((*C.QrdWriter)(w.ptr))
	if result != 0 {
		cErr := C.qrd_last_error()
		if cErr != nil {
			return errors.New(C.GoString(cErr))
		}
		return errors.New("failed to finish writing")
	}
	w.ptr = nil
	return nil
}

// FileReader reads QRD files
type FileReader struct {
	ptr unsafe.Pointer
}

// NewFileReader creates a new file reader
func NewFileReader(path string) (*FileReader, error) {
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	ptr := C.qrd_reader_new(cPath)
	if ptr == nil {
		cErr := C.qrd_last_error()
		if cErr != nil {
			return nil, errors.New(C.GoString(cErr))
		}
		return nil, errors.New("failed to create reader")
	}
	return &FileReader{ptr: unsafe.Pointer(ptr)}, nil
}

// Free releases the reader resources
func (r *FileReader) Free() {
	if r.ptr != nil {
		C.qrd_reader_free((*C.QrdReader)(r.ptr))
		r.ptr = nil
	}
}

// RowCount returns the number of rows in the file
func (r *FileReader) RowCount() uint32 {
	return uint32(C.qrd_reader_row_count((*C.QrdReader)(r.ptr)))
}
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