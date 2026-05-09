//! QRD C FFI — Stable C interface for all language bindings
//!
//! Memory ownership rules:
//! - Functions returning *mut T transfer ownership to caller
//! - Caller MUST call the corresponding _free function
//! - Input *const T are borrowed — caller retains ownership
//! - Returning i32 error codes: 0 = success, negative = error

use qrd_core::schema::{FieldType, Nullability, SchemaBuilder, Schema};
use qrd_core::writer::FileWriter;
use qrd_core::reader::FileReader;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_uint};
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
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
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
    let builder_ref = unsafe { &mut (*builder).0 };
    let name = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };

    let ft = match field_type_from_int(field_type) {
        Ok(ft) => ft,
        Err(e) => {
            set_last_error(e);
            return -1;
        }
    };
    let null = match nullability_from_int(nullability) {
        Ok(n) => n,
        Err(e) => {
            set_last_error(e);
            return -1;
        }
    };

    // Since add_field takes ownership, we need to replace the builder
    // by taking ownership, calling add_field, and putting it back
    let old_builder = std::mem::replace(builder_ref, SchemaBuilder::new());
    match old_builder.add_field(&name, ft, null) {
        Ok(new_builder) => {
            *builder_ref = new_builder;
            0
        }
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn qrd_schema_builder_build(builder: *mut QrdSchemaBuilder) -> *mut QrdSchema {
    let builder = unsafe { Box::from_raw(builder) };
    match builder.0.build() {
        Ok(schema) => Box::into_raw(Box::new(QrdSchema(schema))),
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn qrd_schema_free(ptr: *mut QrdSchema) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

// ============================================================================
// WRITER
// ============================================================================

#[no_mangle]
pub extern "C" fn qrd_writer_new(path: *const c_char, schema: *mut QrdSchema) -> *mut QrdWriter {
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let schema = unsafe { (*schema).0.clone() };

    match FileWriter::new(&path, schema) {
        Ok(w) => Box::into_raw(Box::new(QrdWriter(w))),
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn qrd_writer_free(ptr: *mut QrdWriter) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
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

    let row: Vec<Vec<u8>> = (0..count)
        .map(|i| {
            let ptr = unsafe { *data_ptrs.add(i) };
            let len = unsafe { *data_lens.add(i) } as usize;
            unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
        })
        .collect();

    match writer.write_row(row) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn qrd_writer_finish(ptr: *mut QrdWriter) -> c_int {
    let writer = unsafe { Box::from_raw(ptr) };
    match writer.0.finish() {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
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
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn qrd_reader_free(ptr: *mut QrdReader) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

#[no_mangle]
pub extern "C" fn qrd_reader_row_count(reader: *const QrdReader) -> c_uint {
    unsafe { (*reader).0.row_count() }
}

// ============================================================================
// HELPER FUNCTIONS (PRIVATE)
// ============================================================================

fn field_type_from_int(n: c_int) -> Result<FieldType, String> {
    match n {
        0 => Ok(FieldType::Boolean),
        1 => Ok(FieldType::Int8),
        2 => Ok(FieldType::Int16),
        3 => Ok(FieldType::Int32),
        4 => Ok(FieldType::Int64),
        5 => Ok(FieldType::UInt8),
        6 => Ok(FieldType::UInt16),
        7 => Ok(FieldType::UInt32),
        8 => Ok(FieldType::UInt64),
        9 => Ok(FieldType::Float32),
        10 => Ok(FieldType::Float64),
        11 => Ok(FieldType::Timestamp),
        12 => Ok(FieldType::Date),
        13 => Ok(FieldType::Time),
        14 => Ok(FieldType::String),
        15 => Ok(FieldType::Blob),
        16 => Ok(FieldType::Uuid),
        17 => Ok(FieldType::Decimal),
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
