# 🌐 QRD-SDK Production-Ready — PHASE 4 of 5
## Language Bindings — Python, TypeScript, Go, Java

> Prerequisite: PHASE 1, 2, 3 sudah selesai dan semua tests passing.
> Paste seluruh blok ini ke GitHub Copilot Chat di Codespace.

---

## CONTEXT

Saat ini semua language bindings hanya scaffolding. Target phase ini:
- **FFI layer** (`qrd-ffi`) diperbaiki dan bisa di-compile
- **Python** (PyO3) bisa di-install dan digunakan
- **TypeScript** (WASM) bisa di-bundle dan dijalankan di browser/Node
- **Go** (CGO) bisa di-import dan digunakan
- **Java** (JNA) bisa di-build dengan Maven

---

## TASK 1: Fix FFI Layer (`core/qrd-ffi/src/lib.rs`)

### Masalah Saat Ini

FFI menggunakan `LogicalType` dan `Value` yang tidak ada di public API core:
```rust
// SALAH — types ini tidak match dengan qrd-core
let logical_type = match field_type {
    0 => LogicalType::BOOLEAN,  // ← qrd-core pakai FieldType, bukan LogicalType
```

### Implementasi FFI yang Benar

Buat ulang `qrd-ffi/src/lib.rs` dengan C ABI yang stabil:

```rust
//! QRD C FFI — Stable C interface for all language bindings
//!
//! Memory ownership rules:
//! - Functions returning *mut T transfer ownership to caller
//! - Caller MUST call the corresponding _free function
//! - Input *const T are borrowed — caller retains ownership
//! - Returning i32 error codes: 0 = success, negative = error

use qrd_core::schema::{FieldType, Nullability, SchemaBuilder, Schema};
use qrd_core::writer::{FileWriter, WriterConfig};
use qrd_core::reader::FileReader;
use qrd_core::error::Error;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_double};
use std::ptr;

// ============================================================================
// OPAQUE HANDLES
// ============================================================================

pub struct QrdSchemaBuilder(SchemaBuilder);
pub struct QrdSchema(Schema);
pub struct QrdWriter(FileWriter);
pub struct QrdReader(FileReader);

// Thread safety markers
unsafe impl Send for QrdSchemaBuilder {}
unsafe impl Send for QrdWriter {}
unsafe impl Send for QrdReader {}

// ============================================================================
// ERROR HANDLING
// ============================================================================

thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

fn set_last_error(msg: impl Into<String>) {
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg.into()));
}

/// Get last error message. Returns NULL if no error.
/// Caller must NOT free returned pointer — it's thread-local storage.
#[no_mangle]
pub extern "C" fn qrd_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        match &*e.borrow() {
            Some(msg) => msg.as_ptr() as *const c_char,
            None => ptr::null(),
        }
    })
}

// ============================================================================
// SCHEMA BUILDER
// ============================================================================

#[no_mangle]
pub extern "C" fn qrd_schema_builder_new() -> *mut QrdSchemaBuilder {
    Box::into_raw(Box::new(QrdSchemaBuilder(SchemaBuilder::new())))
}

#[no_mangle]
pub extern "C" fn qrd_schema_builder_free(ptr: *mut QrdSchemaBuilder) {
    if !ptr.is_null() { unsafe { drop(Box::from_raw(ptr)); } }
}

/// field_type values:
///   0=Boolean, 1=Int8, 2=Int16, 3=Int32, 4=Int64,
///   5=UInt8, 6=UInt16, 7=UInt32, 8=UInt64,
///   9=Float32, 10=Float64, 11=Timestamp, 12=Date, 13=Time,
///   14=String, 15=Blob, 16=Uuid, 17=Decimal
///
/// nullability values: 0=Required, 1=Optional, 2=Repeated
#[no_mangle]
pub extern "C" fn qrd_schema_builder_add_field(
    builder: *mut QrdSchemaBuilder,
    name: *const c_char,
    field_type: c_int,
    nullability: c_int,
) -> c_int {
    let builder = unsafe { &mut (*builder).0 };
    let name = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    
    let ft = match field_type_from_int(field_type) {
        Ok(ft) => ft,
        Err(e) => { set_last_error(e); return -1; }
    };
    let null = match nullability_from_int(nullability) {
        Ok(n) => n,
        Err(e) => { set_last_error(e); return -1; }
    };
    
    match builder.add_field(&name, ft, null) {
        Ok(_) => 0,
        Err(e) => { set_last_error(e.to_string()); -1 }
    }
}

#[no_mangle]
pub extern "C" fn qrd_schema_builder_build(builder: *mut QrdSchemaBuilder) -> *mut QrdSchema {
    let builder = unsafe { Box::from_raw(builder) };
    match builder.0.build() {
        Ok(schema) => Box::into_raw(Box::new(QrdSchema(schema))),
        Err(e) => { set_last_error(e.to_string()); ptr::null_mut() }
    }
}

#[no_mangle]
pub extern "C" fn qrd_schema_free(ptr: *mut QrdSchema) {
    if !ptr.is_null() { unsafe { drop(Box::from_raw(ptr)); } }
}

// ============================================================================
// WRITER
// ============================================================================

#[no_mangle]
pub extern "C" fn qrd_writer_new(
    path: *const c_char,
    schema: *mut QrdSchema,
) -> *mut QrdWriter {
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let schema = unsafe { (*schema).0.clone() };
    
    match FileWriter::new(&path, schema) {
        Ok(w) => Box::into_raw(Box::new(QrdWriter(w))),
        Err(e) => { set_last_error(e.to_string()); ptr::null_mut() }
    }
}

#[no_mangle]
pub extern "C" fn qrd_writer_free(ptr: *mut QrdWriter) {
    if !ptr.is_null() { unsafe { drop(Box::from_raw(ptr)); } }
}

/// Write a row. data is array of byte slices.
/// column_count: number of columns
/// data_ptrs: array of pointers to column data
/// data_lens: array of lengths for each column
#[no_mangle]
pub extern "C" fn qrd_writer_write_row(
    writer: *mut QrdWriter,
    column_count: c_uint,
    data_ptrs: *const *const u8,
    data_lens: *const c_uint,
) -> c_int {
    let writer = unsafe { &mut (*writer).0 };
    let count = column_count as usize;
    
    let row: Vec<Vec<u8>> = (0..count).map(|i| {
        let ptr = unsafe { *data_ptrs.add(i) };
        let len = unsafe { *data_lens.add(i) } as usize;
        unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
    }).collect();
    
    match writer.write_row(row) {
        Ok(()) => 0,
        Err(e) => { set_last_error(e.to_string()); -1 }
    }
}

#[no_mangle]
pub extern "C" fn qrd_writer_finish(ptr: *mut QrdWriter) -> c_int {
    let writer = unsafe { Box::from_raw(ptr) };
    match writer.0.finish() {
        Ok(()) => 0,
        Err(e) => { set_last_error(e.to_string()); -1 }
    }
}

// ============================================================================
// READER
// ============================================================================

#[no_mangle]
pub extern "C" fn qrd_reader_new(path: *const c_char) -> *mut QrdReader {
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    match FileReader::new(&path) {
        Ok(r) => Box::into_raw(Box::new(QrdReader(r))),
        Err(e) => { set_last_error(e.to_string()); ptr::null_mut() }
    }
}

#[no_mangle]
pub extern "C" fn qrd_reader_free(ptr: *mut QrdReader) {
    if !ptr.is_null() { unsafe { drop(Box::from_raw(ptr)); } }
}

#[no_mangle]
pub extern "C" fn qrd_reader_row_count(reader: *const QrdReader) -> c_uint {
    unsafe { (*reader).0.row_count() }
}

// Helper functions (private)
fn field_type_from_int(n: c_int) -> Result<FieldType, String> {
    match n {
        0 => Ok(FieldType::Boolean), 1 => Ok(FieldType::Int8),
        2 => Ok(FieldType::Int16),   3 => Ok(FieldType::Int32),
        4 => Ok(FieldType::Int64),   5 => Ok(FieldType::UInt8),
        6 => Ok(FieldType::UInt16),  7 => Ok(FieldType::UInt32),
        8 => Ok(FieldType::UInt64),  9 => Ok(FieldType::Float32),
        10 => Ok(FieldType::Float64), 11 => Ok(FieldType::Timestamp),
        12 => Ok(FieldType::Date),   13 => Ok(FieldType::Time),
        14 => Ok(FieldType::String), 15 => Ok(FieldType::Blob),
        16 => Ok(FieldType::Uuid),   17 => Ok(FieldType::Decimal),
        _ => Err(format!("Unknown field type: {}", n)),
    }
}

fn nullability_from_int(n: c_int) -> Result<Nullability, String> {
    match n {
        0 => Ok(Nullability::Required),
        1 => Ok(Nullability::Optional),
        _ => Err(format!("Unknown nullability: {}", n)),
    }
}
```

### Update `qrd-ffi/Cargo.toml`:
```toml
[package]
name = "qrd-ffi"
version = "0.1.0"
edition = "2021"

[dependencies]
qrd-core = { path = "../qrd-core" }

[lib]
name = "qrd_ffi"
crate-type = ["cdylib", "staticlib"]  # ← PENTING: dua jenis output
```

### Generate C header (`qrd.h`) otomatis:

Tambahkan ke workspace `Cargo.toml`:
```toml
[workspace.metadata.cbindgen]
language = "C"
include_guard = "QRD_H"
```

Buat script `scripts/gen_header.sh`:
```bash
#!/bin/bash
cargo install cbindgen 2>/dev/null || true
cbindgen --config cbindgen.toml --crate qrd-ffi --output sdk/include/qrd.h
```

---

## TASK 2: Python Binding (PyO3)

Update `sdk/python/src/lib.rs`:

```rust
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::FileWriter;
use qrd_core::reader::FileReader;

fn qrd_error_to_py(e: qrd_core::error::Error) -> PyErr {
    PyValueError::new_err(e.to_string())
}

#[pyclass(name = "SchemaBuilder")]
struct PySchemaBuilder {
    inner: SchemaBuilder,
}

#[pymethods]
impl PySchemaBuilder {
    #[new]
    fn new() -> Self {
        PySchemaBuilder { inner: SchemaBuilder::new() }
    }
    
    fn add_field(&mut self, name: &str, field_type: &str, required: bool) -> PyResult<()> {
        let ft = parse_field_type(field_type)?;
        let null = if required { Nullability::Required } else { Nullability::Optional };
        self.inner.add_field(name, ft, null).map_err(qrd_error_to_py)?;
        Ok(())
    }
    
    fn build(&self) -> PyResult<PySchema> {
        let schema = self.inner.clone().build().map_err(qrd_error_to_py)?;
        Ok(PySchema { inner: schema })
    }
}

#[pyclass(name = "Schema")]
#[derive(Clone)]
struct PySchema {
    inner: qrd_core::schema::Schema,
}

#[pymethods]
impl PySchema {
    fn field_count(&self) -> usize { self.inner.fields.len() }
    fn __repr__(&self) -> String { format!("Schema(fields={})", self.inner.fields.len()) }
}

#[pyclass(name = "Writer")]
struct PyWriter {
    inner: Option<FileWriter>,
}

#[pymethods]
impl PyWriter {
    #[new]
    fn new(path: &str, schema: &PySchema) -> PyResult<Self> {
        let writer = FileWriter::new(path, schema.inner.clone()).map_err(qrd_error_to_py)?;
        Ok(PyWriter { inner: Some(writer) })
    }
    
    /// Write a row. columns is list of bytes objects.
    fn write_row(&mut self, columns: Vec<&[u8]>) -> PyResult<()> {
        let writer = self.inner.as_mut().ok_or_else(|| PyValueError::new_err("Writer already finished"))?;
        let row: Vec<Vec<u8>> = columns.iter().map(|c| c.to_vec()).collect();
        writer.write_row(row).map_err(qrd_error_to_py)
    }
    
    fn finish(&mut self) -> PyResult<()> {
        let writer = self.inner.take().ok_or_else(|| PyValueError::new_err("Already finished"))?;
        writer.finish().map_err(qrd_error_to_py)
    }
    
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> { slf }
    fn __exit__(&mut self, _exc_type: PyObject, _exc_val: PyObject, _exc_tb: PyObject) -> bool {
        if self.inner.is_some() {
            let _ = self.finish();
        }
        false
    }
}

#[pyclass(name = "Reader")]
struct PyReader { inner: FileReader }

#[pymethods]
impl PyReader {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let reader = FileReader::new(path).map_err(qrd_error_to_py)?;
        Ok(PyReader { inner: reader })
    }
    
    fn row_count(&self) -> u32 { self.inner.row_count() }
    fn schema(&self) -> PySchema { PySchema { inner: self.inner.schema().clone() } }
}

fn parse_field_type(s: &str) -> PyResult<FieldType> {
    match s.to_uppercase().as_str() {
        "BOOLEAN" | "BOOL" => Ok(FieldType::Boolean),
        "INT8" | "I8" => Ok(FieldType::Int8),
        "INT16" | "I16" => Ok(FieldType::Int16),
        "INT32" | "I32" | "INT" => Ok(FieldType::Int32),
        "INT64" | "I64" | "LONG" => Ok(FieldType::Int64),
        "FLOAT32" | "F32" | "FLOAT" => Ok(FieldType::Float32),
        "FLOAT64" | "F64" | "DOUBLE" => Ok(FieldType::Float64),
        "STRING" | "STR" | "UTF8" => Ok(FieldType::String),
        "BLOB" | "BYTES" | "BINARY" => Ok(FieldType::Blob),
        "TIMESTAMP" | "TS" => Ok(FieldType::Timestamp),
        _ => Err(PyValueError::new_err(format!("Unknown field type: {}", s))),
    }
}

#[pymodule]
fn qrd(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySchemaBuilder>()?;
    m.add_class::<PySchema>()?;
    m.add_class::<PyWriter>()?;
    m.add_class::<PyReader>()?;
    Ok(())
}
```

### Update `sdk/python/Cargo.toml`:
```toml
[package]
name = "qrd-python"
version = "0.1.0"
edition = "2021"

[dependencies]
pyo3 = { version = "0.21", features = ["extension-module"] }
qrd-core = { path = "../../core/qrd-core" }

[lib]
name = "qrd"
crate-type = ["cdylib"]
```

### Update `sdk/python/setup.py` → Gunakan `maturin`:
```toml
# sdk/python/pyproject.toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "qrd-sdk"
version = "0.1.0"
description = "QRD columnar binary format SDK"

[tool.maturin]
features = ["pyo3/extension-module"]
```

### Python test (`sdk/python/tests/test_qrd.py`):
```python
import qrd
import struct

def test_basic_write_read():
    schema = (qrd.SchemaBuilder()
        .add_field("id", "INT64")
        .add_field("name", "STRING")
        .build())
    
    with qrd.Writer("/tmp/test.qrd", schema) as w:
        for i in range(100):
            id_bytes = struct.pack("<q", i)       # i64 LE
            name_bytes = f"user_{i}".encode()
            name_bytes = struct.pack("<I", len(name_bytes)) + name_bytes  # length-prefix
            w.write_row([id_bytes, name_bytes])
    
    reader = qrd.Reader("/tmp/test.qrd")
    assert reader.row_count() == 100
```

---

## TASK 3: TypeScript/WASM Binding

### Update `core/qrd-wasm/src/lib.rs`:
```rust
use wasm_bindgen::prelude::*;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::StreamingWriter;
use std::io::Cursor;

#[wasm_bindgen]
pub struct QrdSchemaBuilder {
    inner: SchemaBuilder,
}

#[wasm_bindgen]
impl QrdSchemaBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        QrdSchemaBuilder { inner: SchemaBuilder::new() }
    }
    
    pub fn add_field(&mut self, name: &str, field_type: &str) -> Result<(), JsValue> {
        let ft = parse_field_type(field_type).map_err(|e| JsValue::from_str(&e))?;
        self.inner.add_field(name, ft, Nullability::Required)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }
    
    pub fn build(self) -> Result<QrdSchema, JsValue> {
        let schema = self.inner.build().map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(QrdSchema { inner: schema })
    }
}

#[wasm_bindgen]
pub struct QrdSchema {
    inner: qrd_core::schema::Schema,
}

/// In-memory writer — returns bytes when finished
#[wasm_bindgen]
pub struct QrdMemWriter {
    buffer: Vec<u8>,
    writer: Option<StreamingWriter<Cursor<Vec<u8>>>>,
}

#[wasm_bindgen]
impl QrdMemWriter {
    #[wasm_bindgen(constructor)]
    pub fn new(schema: &QrdSchema) -> Result<Self, JsValue> {
        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let writer = StreamingWriter::new(cursor, schema.inner.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(QrdMemWriter { buffer: Vec::new(), writer: Some(writer) })
    }
    
    pub fn write_row(&mut self, columns: Vec<js_sys::Uint8Array>) -> Result<(), JsValue> {
        let writer = self.writer.as_mut().ok_or("Writer finished")?;
        let row: Vec<Vec<u8>> = columns.iter().map(|a| a.to_vec()).collect();
        writer.write_row(row).map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
    pub fn finish(mut self) -> Result<js_sys::Uint8Array, JsValue> {
        let writer = self.writer.take().ok_or("Already finished")?;
        let cursor = writer.finish_into_inner()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let bytes = cursor.into_inner();
        Ok(js_sys::Uint8Array::from(bytes.as_slice()))
    }
}
```

### Update `sdk/typescript/src/index.ts`:
```typescript
import init, { QrdSchemaBuilder, QrdMemWriter } from '../pkg/qrd_wasm';

export async function createQrdFile(
  fields: Array<{ name: string; type: string }>,
  rows: Array<Array<Uint8Array>>
): Promise<Uint8Array> {
  await init();
  
  const builder = new QrdSchemaBuilder();
  for (const field of fields) {
    builder.addField(field.name, field.type);
  }
  const schema = builder.build();
  
  const writer = new QrdMemWriter(schema);
  for (const row of rows) {
    writer.writeRow(row);
  }
  
  return writer.finish();
}
```

---

## TASK 4: Go Binding Update

Update `sdk/go/qrd.go` agar link ke real compiled library:
```go
package qrd

/*
#cgo LDFLAGS: -L${SRCDIR}/../../target/release -lqrd_ffi
#include "../../sdk/include/qrd.h"
#include <stdlib.h>
*/
import "C"
import (
    "errors"
    "unsafe"
)

type SchemaBuilder struct {
    ptr *C.QrdSchemaBuilder
}

func NewSchemaBuilder() *SchemaBuilder {
    return &SchemaBuilder{ptr: C.qrd_schema_builder_new()}
}

func (sb *SchemaBuilder) Free() {
    C.qrd_schema_builder_free(sb.ptr)
}

func (sb *SchemaBuilder) AddField(name string, fieldType int, nullable bool) error {
    cName := C.CString(name)
    defer C.free(unsafe.Pointer(cName))
    nullability := C.int(0)
    if nullable { nullability = 1 }
    if C.qrd_schema_builder_add_field(sb.ptr, cName, C.int(fieldType), nullability) != 0 {
        return errors.New(C.GoString(C.qrd_last_error()))
    }
    return nil
}
```

---

## VALIDASI PHASE 4

```bash
# 1. FFI compiles
cargo build --package qrd-ffi --release 2>&1 | grep -E "error|Finished"

# 2. Python binding
cd sdk/python
pip install maturin
maturin develop 2>&1 | tail -5
python -c "import qrd; print('Python binding OK:', dir(qrd))"
python tests/test_qrd.py

# 3. WASM binding
cargo install wasm-pack
cd core/qrd-wasm
wasm-pack build --target web 2>&1 | tail -5

# 4. Go binding (perlu libqrd_ffi.so di PATH)
cd sdk/go
go test ./... 2>&1

# 5. Java build
cd sdk/java
mvn compile test 2>&1 | grep -E "BUILD|ERROR|Tests run"
```

**Expected:**
```
Python binding OK: ['Reader', 'Schema', 'SchemaBuilder', 'Writer']
[INFO] BUILD SUCCESS
WASM pkg generated at: pkg/
```

---

## CONSTRAINT

- FFI must be `#[no_mangle]` extern "C" — tidak boleh mangling
- Memory ownership HARUS jelas: setiap `*mut T` yang dikembalikan HARUS ada `_free` function
- Python binding harus support context manager (`with` statement)
- WASM binding tidak boleh pakai `std::fs` (tidak ada filesystem di browser)
- Go binding harus ada `Free()` method untuk semua opaque handles
- Java: gunakan JNA (bukan JNI) untuk simplicity — map langsung ke C FFI

**Lanjut ke PHASE 5 setelah semua validasi passing.**
