//! QRD FFI - C Foreign Function Interface

use qrd_core::prelude::*;
use std::cell::RefCell;
use std::ffi::CStr;
use std::io::{self, Write};
use std::os::raw::c_char;
use std::rc::Rc;

#[derive(Clone)]
struct SharedVecWriter {
    buffer: Rc<RefCell<Vec<u8>>>,
}

impl SharedVecWriter {
    fn new() -> Self {
        Self {
            buffer: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn bytes(&self) -> Vec<u8> {
        self.buffer.borrow().clone()
    }
}

impl Write for SharedVecWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// FFI wrapper for schema.
#[repr(C)]
pub struct FFISchema {
    inner: Schema,
}

/// FFI wrapper for writer.
#[repr(C)]
pub struct FFIWriter {
    inner: Option<qrd_core::writer::StreamingWriter<SharedVecWriter>>,
    buffer: SharedVecWriter,
}

/// FFI wrapper for reader.
#[repr(C)]
pub struct FFIReader {
    buffer: Vec<u8>,
}

/// FFI wrapper for row.
#[repr(C)]
pub struct FFIRow {
    values: Vec<Vec<u8>>,
}

fn field_type_from_ffi(value: i32) -> Option<FieldType> {
    match value {
        0 => Some(FieldType::Boolean),
        1 => Some(FieldType::Int8),
        2 => Some(FieldType::Int16),
        3 => Some(FieldType::Int32),
        4 => Some(FieldType::Int64),
        5 => Some(FieldType::UInt8),
        6 => Some(FieldType::UInt16),
        7 => Some(FieldType::UInt32),
        8 => Some(FieldType::UInt64),
        9 => Some(FieldType::Float32),
        10 => Some(FieldType::Float64),
        11 => Some(FieldType::Timestamp),
        12 => Some(FieldType::Date),
        13 => Some(FieldType::Time),
        14 => Some(FieldType::Duration),
        15 => Some(FieldType::String),
        16 => Some(FieldType::Enum),
        17 => Some(FieldType::Uuid),
        18 => Some(FieldType::Blob),
        19 => Some(FieldType::Decimal),
        _ => None,
    }
}

fn nullability_from_ffi(value: i32) -> Option<Nullability> {
    match value {
        0 => Some(Nullability::Required),
        1 => Some(Nullability::Optional),
        2 => Some(Nullability::Repeated),
        _ => None,
    }
}

/// FFI function to create schema.
#[no_mangle]
pub extern "C" fn qrd_schema_new_ffi() -> *mut FFISchema {
    let schema = FFISchema {
        inner: Schema {
            fields: vec![],
            schema_id: 0,
        },
    };
    Box::into_raw(Box::new(schema))
}

/// FFI function to free schema.
#[no_mangle]
pub extern "C" fn qrd_schema_free_ffi(schema: *mut FFISchema) {
    if !schema.is_null() {
        unsafe {
            let _ = Box::from_raw(schema);
        }
    }
}

/// FFI function to add field to schema.
#[no_mangle]
pub extern "C" fn qrd_schema_add_field_ffi(
    schema: *mut FFISchema,
    name: *const c_char,
    field_type: i32,
    nullability: i32,
    _metadata: *const c_char,
) -> i32 {
    if schema.is_null() || name.is_null() {
        return -1;
    }

    let field_type = match field_type_from_ffi(field_type) {
        Some(value) => value,
        None => return -1,
    };
    let nullability = match nullability_from_ffi(nullability) {
        Some(value) => value,
        None => return -1,
    };

    unsafe {
        let schema_ref = &mut *schema;
        let name_str = CStr::from_ptr(name).to_string_lossy().to_string();

        let mut builder = SchemaBuilder::new();
        for field in &schema_ref.inner.fields {
            builder = match builder.add_field(&field.name, field.field_type, field.nullability) {
                Ok(next) => next,
                Err(_) => return -1,
            };
        }

        builder = match builder.add_field(name_str, field_type, nullability) {
            Ok(next) => next,
            Err(_) => return -1,
        };

        match builder.build() {
            Ok(new_schema) => {
                schema_ref.inner = new_schema;
                0
            }
            Err(_) => -1,
        }
    }
}

/// FFI function to get schema ID.
#[no_mangle]
pub extern "C" fn qrd_schema_id_ffi(schema: *const FFISchema) -> u64 {
    if schema.is_null() {
        return 0;
    }
    unsafe { (*schema).inner.schema_id as u64 }
}

/// FFI function to get field count.
#[no_mangle]
pub extern "C" fn qrd_schema_field_count_ffi(schema: *const FFISchema) -> usize {
    if schema.is_null() {
        return 0;
    }
    unsafe { (*schema).inner.fields.len() }
}

/// FFI function to create writer.
#[no_mangle]
pub extern "C" fn qrd_writer_new_ffi(schema: *const FFISchema) -> *mut FFIWriter {
    if schema.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let buffer = SharedVecWriter::new();
        let writer = match qrd_core::writer::StreamingWriter::new(buffer.clone(), (*schema).inner.clone()) {
            Ok(writer) => writer,
            Err(_) => return std::ptr::null_mut(),
        };

        let ffi_writer = FFIWriter {
            inner: Some(writer),
            buffer,
        };
        Box::into_raw(Box::new(ffi_writer))
    }
}

/// FFI function to free writer.
#[no_mangle]
pub extern "C" fn qrd_writer_free_ffi(writer: *mut FFIWriter) {
    if !writer.is_null() {
        unsafe {
            let _ = Box::from_raw(writer);
        }
    }
}

/// FFI function to write row.
#[no_mangle]
pub extern "C" fn qrd_writer_write_row_ffi(writer: *mut FFIWriter, row: *const FFIRow) -> i32 {
    if writer.is_null() || row.is_null() {
        return -1;
    }

    unsafe {
        let writer_ref = &mut *writer;
        if let Some(ref mut w) = writer_ref.inner {
            let row_ref = &*row;
            match w.write_row(row_ref.values.clone()) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

/// FFI function to finish writing.
#[no_mangle]
pub extern "C" fn qrd_writer_finish_ffi(writer: *mut FFIWriter, data: *mut *mut u8, size: *mut usize) -> i32 {
    if writer.is_null() || data.is_null() || size.is_null() {
        return -1;
    }

    unsafe {
        let writer_ref = &mut *writer;
        if let Some(w) = writer_ref.inner.take() {
            match w.finish() {
                Ok(()) => {
                    let buffer = writer_ref.buffer.bytes();
                    let len = buffer.len();
                    let ptr = buffer.as_ptr() as *mut u8;

                    std::mem::forget(buffer);

                    *data = ptr;
                    *size = len;
                    0
                }
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

/// FFI function to create reader.
#[no_mangle]
pub extern "C" fn qrd_reader_new_ffi(data: *const u8, size: usize) -> *mut FFIReader {
    if data.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let slice = std::slice::from_raw_parts(data, size);
        let ffi_reader = FFIReader {
            buffer: slice.to_vec(),
        };
        Box::into_raw(Box::new(ffi_reader))
    }
}

/// FFI function to free reader.
#[no_mangle]
pub extern "C" fn qrd_reader_free_ffi(reader: *mut FFIReader) {
    if !reader.is_null() {
        unsafe {
            let _ = Box::from_raw(reader);
        }
    }
}
