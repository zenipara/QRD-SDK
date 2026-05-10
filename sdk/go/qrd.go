// Package qrd provides Go bindings for the QRD columnar binary format.
package qrd

/*
#cgo LDFLAGS: -L${SRCDIR}/../../target/release -Wl,-rpath,$ORIGIN/../../target/release -lqrd_ffi
#include <stdlib.h>
#include "qrd.h"
*/
import "C"

import (
	"encoding/binary"
	"errors"
	"fmt"
	"math"
	"runtime"
	"unsafe"
)

// FieldType represents the logical type of a schema field.
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
	FieldTypeUuid
	FieldTypeBlob
	FieldTypeDecimal
)

// Nullability represents whether a field can be null.
type Nullability int

const (
	NullabilityRequired Nullability = iota
	NullabilityOptional
	NullabilityRepeated
)

// FileWriter writes QRD files.
type FileWriter struct {
	ptr unsafe.Pointer
}

// FileReader reads QRD files.
type FileReader struct {
	ptr unsafe.Pointer
}

// SchemaBuilder builds QRD schemas.
type SchemaBuilder struct {
	ptr unsafe.Pointer
	err error
}

func newSchemaBuilder() *SchemaBuilder {
	ptr := C.qrd_schema_builder_new()
	if ptr == nil {
		return &SchemaBuilder{err: errors.New("failed to create schema")}
	}

	sb := &SchemaBuilder{ptr: unsafe.Pointer(ptr)}
	runtime.SetFinalizer(sb, func(sb *SchemaBuilder) {
		sb.Free()
	})
	return sb
}

// NewSchemaBuilder creates a new schema builder.
func NewSchemaBuilder() *SchemaBuilder {
	return newSchemaBuilder()
}

// Free releases the schema builder resources.
func (sb *SchemaBuilder) Free() {
	if sb == nil || sb.ptr == nil {
		return
	}
	C.qrd_schema_builder_free((*C.QrdSchemaBuilder)(sb.ptr))
	sb.ptr = nil
}

// Close is a semantic alias for Free and helps Go users manage lifetime.
func (sb *SchemaBuilder) Close() {
	sb.Free()
}

func (sb *SchemaBuilder) AddField(name string, fieldType FieldType, nullability Nullability, metadata string) *SchemaBuilder {
	if sb == nil || sb.ptr == nil || sb.err != nil {
		return sb
	}

	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	cMetadata := C.CString(metadata)
	defer C.free(unsafe.Pointer(cMetadata))

	result := C.qrd_schema_builder_add_field(
		(*C.QrdSchemaBuilder)(sb.ptr),
		cName,
		C.int(fieldType),
		C.int(nullability),
	)
	if result != 0 {
		sb.err = errors.New("failed to add field to schema")
	}
	return sb
}

// Build finalizes the schema.
func (sb *SchemaBuilder) Build() (*Schema, error) {
	if sb == nil {
		return nil, errors.New("schema builder is nil")
	}
	if sb.err != nil {
		return nil, sb.err
	}
	if sb.ptr == nil {
		return nil, errors.New("schema builder is not initialized")
	}

	built := C.qrd_schema_builder_build((*C.QrdSchemaBuilder)(sb.ptr))
	if built == nil {
		return nil, errors.New("failed to build schema")
	}

	schema := &Schema{ptr: built}
	sb.ptr = nil
	runtime.SetFinalizer(schema, func(s *Schema) {
		s.Free()
	})
	return schema, nil
}

// Schema represents a QRD schema.
type Schema struct {
	ptr *C.QrdSchema
}

// Free releases the schema resources.
func (s *Schema) Free() {
	if s == nil || s.ptr == nil {
		return
	}
	C.qrd_schema_free((*C.QrdSchema)(s.ptr))
	s.ptr = nil
}

// Close is a semantic alias for Free and provides idiomatic cleanup.
func (s *Schema) Close() {
    s.Free()
}

// FieldCount returns the number of fields in the schema.
func (s *Schema) FieldCount() int {
	if s == nil || s.ptr == nil {
		return 0
	}
	return int(C.qrd_schema_field_count((*C.QrdSchema)(s.ptr)))
}

// FileWriter writes QRD files.
type FileWriter struct {
	ptr unsafe.Pointer
}

// NewFileWriter creates a new file writer.
func NewFileWriter(schema *Schema) (*FileWriter, error) {
	if schema == nil || schema.ptr == nil {
		return nil, errors.New("schema is nil")
	}

	ptr := C.qrd_writer_new((*C.QrdSchema)(schema.ptr))
	if ptr == nil {
		return nil, errors.New("failed to create writer")
	}

	writer := &FileWriter{ptr: unsafe.Pointer(ptr)}
	runtime.SetFinalizer(writer, func(w *FileWriter) {
		w.Free()
	})
	return writer, nil
}

// Free releases the writer resources.
func (w *FileWriter) Free() {
	if w == nil || w.ptr == nil {
		return
	}
	C.qrd_writer_free((*C.QrdWriter)(w.ptr))
	w.ptr = nil
}

// Close releases writer resources.
func (w *FileWriter) Close() {
    w.Free()
}

func encodeColumnValue(value interface{}) ([]byte, error) {
    switch v := value.(type) {
    case nil:
        return []byte{}, nil
    case bool:
        if v {
            return []byte{1}, nil
        }
        return []byte{0}, nil
    case int:
        var buf [8]byte
        binary.LittleEndian.PutUint64(buf[:], uint64(int64(v)))
        return buf[:], nil
    case int8:
        return []byte{byte(v)}, nil
    case int16:
        var buf [2]byte
        binary.LittleEndian.PutUint16(buf[:], uint16(v))
        return buf[:], nil
    case int32:
        var buf [4]byte
        binary.LittleEndian.PutUint32(buf[:], uint32(v))
        return buf[:], nil
    case int64:
        var buf [8]byte
        binary.LittleEndian.PutUint64(buf[:], uint64(v))
        return buf[:], nil
    case uint:
        var buf [8]byte
        binary.LittleEndian.PutUint64(buf[:], uint64(v))
        return buf[:], nil
    case uint8:
        return []byte{byte(v)}, nil
    case uint16:
        var buf [2]byte
        binary.LittleEndian.PutUint16(buf[:], v)
        return buf[:], nil
    case uint32:
        var buf [4]byte
        binary.LittleEndian.PutUint32(buf[:], v)
        return buf[:], nil
    case uint64:
        var buf [8]byte
        binary.LittleEndian.PutUint64(buf[:], v)
        return buf[:], nil
    case float32:
        var buf [4]byte
        binary.LittleEndian.PutUint32(buf[:], math.Float32bits(v))
        return buf[:], nil
    case float64:
        var buf [8]byte
        binary.LittleEndian.PutUint64(buf[:], math.Float64bits(v))
        return buf[:], nil
    case string:
        payload := []byte(v)
        serialized := make([]byte, 4+len(payload))
        binary.LittleEndian.PutUint32(serialized[:4], uint32(len(payload)))
        copy(serialized[4:], payload)
        return serialized, nil
    case []byte:
        serialized := make([]byte, 4+len(v))
        binary.LittleEndian.PutUint32(serialized[:4], uint32(len(v)))
        copy(serialized[4:], v)
        return serialized, nil
    default:
        return nil, fmt.Errorf("unsupported column type %T", value)
    }
}

// WriteRow writes a row of column values.
func (w *FileWriter) WriteRow(columns []interface{}) error {
	if w == nil || w.ptr == nil {
		return errors.New("writer is nil")
	}

	cRow := C.qrd_row_new()
	if cRow == nil {
		return errors.New("failed to create row")
	}
	defer C.qrd_row_free(cRow)

	for _, column := range columns {
		encoded, err := encodeColumnValue(column)
		if err != nil {
			return err
		}

		var dataPtr *C.uint8_t
		if len(encoded) > 0 {
			dataPtr = (*C.uint8_t)(unsafe.Pointer(&encoded[0]))
		}

		if C.qrd_row_add_bytes(cRow, dataPtr, C.size_t(len(encoded))) != 0 {
			return errors.New("failed to add column to row")
		}
	}

	if C.qrd_writer_write_row((*C.QrdWriter)(w.ptr), cRow) != 0 {
		return errors.New("failed to write row")
	}
	return nil
}

// Finish finishes writing the file and returns the encoded bytes.
func (w *FileWriter) Finish() ([]byte, error) {
	if w == nil || w.ptr == nil {
		return nil, errors.New("writer is nil")
	}

	var cData *C.uint8_t
	var cSize C.size_t
	if C.qrd_writer_finish((*C.QrdWriter)(w.ptr), &cData, &cSize) != 0 {
		return nil, errors.New("failed to finish writing")
	}

	defer w.Free()

	if cData != nil && cSize > 0 {
		defer C.free(unsafe.Pointer(cData))
		return C.GoBytes(unsafe.Pointer(cData), C.int(cSize)), nil
	}

	return []byte{}, nil
}

// NewFileReader creates a new file reader from encoded bytes.
func NewFileReader(data []byte) (*FileReader, error) {
	if len(data) == 0 {
		return nil, errors.New("data is empty")
	}

	readerPtr := C.qrd_reader_new((*C.uint8_t)(unsafe.Pointer(&data[0])), C.size_t(len(data)))
	if readerPtr == nil {
		return nil, errors.New("failed to create reader")
	}

	cSchema := C.qrd_reader_schema((*C.QrdReader)(readerPtr))
	if cSchema == nil {
		C.qrd_reader_free((*C.QrdReader)(readerPtr))
		return nil, errors.New("failed to get schema")
	}

	schema := &Schema{ptr: cSchema}
	runtime.SetFinalizer(schema, func(s *Schema) {
		s.Free()
	})

	reader := &FileReader{
		ptr:    unsafe.Pointer(readerPtr),
		schema: schema,
	}
	runtime.SetFinalizer(reader, func(r *FileReader) {
		r.Free()
	})

	return reader, nil
}

// Free releases reader resources.
func (r *FileReader) Free() {
	if r == nil || r.ptr == nil {
		return
	}
	C.qrd_reader_free((*C.QrdReader)(r.ptr))
	r.ptr = nil
}

// Close releases reader resources.
func (r *FileReader) Close() {
    r.Free()
}

// RowCount returns the number of rows in the file.
func (r *FileReader) RowCount() uint64 {
	if r == nil || r.ptr == nil {
		return 0
	}
	return uint64(C.qrd_reader_row_count((*C.QrdReader)(r.ptr)))
}

// ReadRow reads the next row.
func (r *FileReader) ReadRow() ([]interface{}, error) {
	if r == nil || r.ptr == nil {
		return nil, errors.New("reader is nil")
	}

	cRow := C.qrd_reader_read_row((*C.QrdReader)(r.ptr))
	if cRow == nil {
		return nil, nil
	}
	defer C.qrd_row_free(cRow)

	fieldCount := int(C.qrd_row_field_count(cRow))
	return make([]interface{}, fieldCount), nil
}

// ReadAllRows reads all rows at once.
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