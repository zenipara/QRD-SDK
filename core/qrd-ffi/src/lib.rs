//! QRD FFI - C Foreign Function Interface

use qrd_core::prelude::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// FFI wrapper for schema
#[repr(C)]
pub struct FFISchema {
    inner: Schema,
}

/// FFI wrapper for writer
#[repr(C)]
pub struct FFIWriter {
    inner: Option<qrd_core::writer::FileWriter<std::io::Cursor<Vec<u8>>>>,
}

/// FFI wrapper for reader
#[repr(C)]
pub struct FFIReader {
    inner: Option<qrd_core::reader::FileReader<std::io::Cursor<Vec<u8>>>>,
}

/// FFI wrapper for row
#[repr(C)]
pub struct FFIRow {
    values: Vec<qrd_core::schema::Value>,
}

/// FFI function to create schema
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

/// FFI function to free schema
#[no_mangle]
pub extern "C" fn qrd_schema_free_ffi(schema: *mut FFISchema) {
    if !schema.is_null() {
        unsafe {
            let _ = Box::from_raw(schema);
        }
    }
}

/// FFI function to add field to schema
#[no_mangle]
pub extern "C" fn qrd_schema_add_field_ffi(
    schema: *mut FFISchema,
    name: *const c_char,
    field_type: i32,
    nullability: i32,
    metadata: *const c_char,
) -> i32 {
    if schema.is_null() || name.is_null() {
        return -1;
    }

    unsafe {
        let schema_ref = &mut *schema;
        let name_str = CStr::from_ptr(name).to_string_lossy();

        let logical_type = match field_type {
            0 => LogicalType::BOOLEAN,
            1 => LogicalType::INT8,
            2 => LogicalType::INT16,
            3 => LogicalType::INT32,
            4 => LogicalType::INT64,
            5 => LogicalType::UINT8,
            6 => LogicalType::UINT16,
            7 => LogicalType::UINT32,
            8 => LogicalType::UINT64,
            9 => LogicalType::FLOAT32,
            10 => LogicalType::FLOAT64,
            11 => LogicalType::TIMESTAMP,
            12 => LogicalType::DATE,
            13 => LogicalType::TIME,
            14 => LogicalType::DURATION,
            15 => LogicalType::STRING,
            16 => LogicalType::ENUM,
            17 => LogicalType::UUID,
            18 => LogicalType::BLOB,
            19 => LogicalType::DECIMAL,
            _ => return -1,
        };

        let nullability_type = match nullability {
            0 => Nullability::REQUIRED,
            1 => Nullability::OPTIONAL,
            2 => Nullability::REPEATED,
            _ => return -1,
        };

        let metadata_str = if !metadata.is_null() {
            Some(CStr::from_ptr(metadata).to_string_lossy().to_string())
        } else {
            None
        };

        // Create a temporary schema builder to add the field
        let mut builder = qrd_core::schema::SchemaBuilder::new();
        for field in &schema_ref.inner.fields {
            builder = builder.add_field(&field.name, field.logical_type, field.nullability, field.metadata.as_deref());
        }
        builder = builder.add_field(&name_str, logical_type, nullability_type, metadata_str.as_deref());

        match builder.build() {
            Ok(new_schema) => {
                schema_ref.inner = new_schema;
                0
            }
            Err(_) => -1,
        }
    }
}

/// FFI function to get schema ID
#[no_mangle]
pub extern "C" fn qrd_schema_id_ffi(schema: *const FFISchema) -> u64 {
    if schema.is_null() {
        return 0;
    }
    unsafe { (*schema).inner.schema_id }
}

/// FFI function to get field count
#[no_mangle]
pub extern "C" fn qrd_schema_field_count_ffi(schema: *const FFISchema) -> usize {
    if schema.is_null() {
        return 0;
    }
    unsafe { (*schema).inner.fields.len() }
}

/// FFI function to create writer
#[no_mangle]
pub extern "C" fn qrd_writer_new_ffi(schema: *const FFISchema) -> *mut FFIWriter {
    if schema.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let buffer = Vec::new();
        let cursor = std::io::Cursor::new(buffer);
        let writer = match qrd_core::writer::FileWriter::new(cursor, (*schema).inner.clone()) {
            Ok(w) => w,
            Err(_) => return std::ptr::null_mut(),
        };

        let ffi_writer = FFIWriter {
            inner: Some(writer),
        };
        Box::into_raw(Box::new(ffi_writer))
    }
}

/// FFI function to free writer
#[no_mangle]
pub extern "C" fn qrd_writer_free_ffi(writer: *mut FFIWriter) {
    if !writer.is_null() {
        unsafe {
            let _ = Box::from_raw(writer);
        }
    }
}

/// FFI function to write row
#[no_mangle]
pub extern "C" fn qrd_writer_write_row_ffi(writer: *mut FFIWriter, row: *const FFIRow) -> i32 {
    if writer.is_null() || row.is_null() {
        return -1;
    }

    unsafe {
        let writer_ref = &mut *writer;
        if let Some(ref mut w) = writer_ref.inner {
            let row_ref = &*row;
            // Convert FFIRow to the internal row format
            // This is simplified - in practice you'd need proper conversion
            let internal_row = row_ref.values.clone();
            match w.write_row(&internal_row) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

/// FFI function to finish writing
#[no_mangle]
pub extern "C" fn qrd_writer_finish_ffi(writer: *mut FFIWriter, data: *mut *mut u8, size: *mut usize) -> i32 {
    if writer.is_null() || data.is_null() || size.is_null() {
        return -1;
    }

    unsafe {
        let writer_ref = &mut *writer;
        if let Some(w) = writer_ref.inner.take() {
            match w.finish() {
                Ok(cursor) => {
                    let buffer = cursor.into_inner();
                    let len = buffer.len();
                    let ptr = buffer.as_ptr() as *mut u8;

                    // Prevent the buffer from being deallocated
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

/// FFI function to create reader
#[no_mangle]
pub extern "C" fn qrd_reader_new_ffi(data: *const u8, size: usize) -> *mut FFIReader {
    if data.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let slice = std::slice::from_raw_parts(data, size);
        let buffer = slice.to_vec();
        let cursor = std::io::Cursor::new(buffer);
        let reader = match qrd_core::reader::FileReader::new(cursor) {
            Ok(r) => r,
            Err(_) => return std::ptr::null_mut(),
        };

        let ffi_reader = FFIReader {
            inner: Some(reader),
        };
        Box::into_raw(Box::new(ffi_reader))
    }
}

/// FFI function to free reader
#[no_mangle]
pub extern "C" fn qrd_reader_free_ffi(reader: *mut FFIReader) {
    if !reader.is_null() {
        unsafe {
            let _ = Box::from_raw(reader);
        }
    }
}

/// FFI function to get reader schema
#[no_mangle]
pub extern "C" fn qrd_reader_schema_ffi(reader: *const FFIReader) -> *mut FFISchema {
    if reader.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let reader_ref = &*reader;
        if let Some(ref r) = reader_ref.inner {
            let schema = r.schema().clone();
            let ffi_schema = FFISchema { inner: schema };
            Box::into_raw(Box::new(ffi_schema))
        } else {
            std::ptr::null_mut()
        }
    }
}

/// FFI function to get row count
#[no_mangle]
pub extern "C" fn qrd_reader_row_count_ffi(reader: *const FFIReader) -> u64 {
    if reader.is_null() {
        return 0;
    }

    unsafe {
        let reader_ref = &*reader;
        if let Some(ref r) = reader_ref.inner {
            r.row_count()
        } else {
            0
        }
    }
}

/// FFI function to read row
#[no_mangle]
pub extern "C" fn qrd_reader_read_row_ffi(reader: *mut FFIReader) -> *mut FFIRow {
    if reader.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let reader_ref = &mut *reader;
        if let Some(ref mut r) = reader_ref.inner {
            match r.read_row() {
                Ok(Some(row)) => {
                    let ffi_row = FFIRow {
                        values: row,
                    };
                    Box::into_raw(Box::new(ffi_row))
                }
                _ => std::ptr::null_mut(),
            }
        } else {
            std::ptr::null_mut()
        }
    }
}

/// FFI function to create row
#[no_mangle]
pub extern "C" fn qrd_row_new_ffi() -> *mut FFIRow {
    let row = FFIRow {
        values: Vec::new(),
    };
    Box::into_raw(Box::new(row))
}

/// FFI function to free row
#[no_mangle]
pub extern "C" fn qrd_row_free_ffi(row: *mut FFIRow) {
    if !row.is_null() {
        unsafe {
            let _ = Box::from_raw(row);
        }
    }
}

/// FFI function to get field count
#[no_mangle]
pub extern "C" fn qrd_row_field_count_ffi(row: *const FFIRow) -> usize {
    if row.is_null() {
        return 0;
    }
    unsafe { (*row).values.len() }
}

/// FFI function to add int64 to row
#[no_mangle]
pub extern "C" fn qrd_row_add_int64_ffi(row: *mut FFIRow, value: i64) -> i32 {
    if row.is_null() {
        return -1;
    }
    unsafe {
        (*row).values.push(qrd_core::schema::Value::Int64(value));
    }
    0
}

/// FFI function to add float64 to row
#[no_mangle]
pub extern "C" fn qrd_row_add_float64_ffi(row: *mut FFIRow, value: f64) -> i32 {
    if row.is_null() {
        return -1;
    }
    unsafe {
        (*row).values.push(qrd_core::schema::Value::Float64(value));
    }
    0
}

/// FFI function to add string to row
#[no_mangle]
pub extern "C" fn qrd_row_add_string_ffi(row: *mut FFIRow, value: *const c_char) -> i32 {
    if row.is_null() || value.is_null() {
        return -1;
    }
    unsafe {
        let str_value = CStr::from_ptr(value).to_string_lossy().to_string();
        (*row).values.push(qrd_core::schema::Value::String(str_value));
    }
    0
}
