//! QRD FFI - C Foreign Function Interface

use qrd_core::prelude::*;
use qrd_core::reader::FileReader as CoreFileReader;
use std::cell::RefCell;
use std::ffi::c_void;
use std::ffi::CStr;
use std::fs;
use std::io::{self, Write};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::rc::Rc;
use std::slice;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEMP_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
}

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
    schema: Schema,
    rows: Vec<Vec<u8>>,
    row_index: usize,
}

/// FFI wrapper for row.
#[repr(C)]
pub struct FFIRow {
    values: Vec<Vec<u8>>,
}

fn temp_reader_path() -> PathBuf {
    let pid = std::process::id();
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("qrd_ffi_reader_{}_{}.qrd", pid, counter))
}

fn split_row_bytes(row_bytes: &[u8], schema: &Schema) -> Option<Vec<Vec<u8>>> {
    let mut values = Vec::with_capacity(schema.fields.len());
    let mut offset = 0usize;

    for field in &schema.fields {
        match field.field_type.fixed_size() {
            Some(size) => {
                let end = offset.checked_add(size)?;
                let slice = row_bytes.get(offset..end)?;
                values.push(slice.to_vec());
                offset = end;
            }
            None => {
                let len_end = offset.checked_add(4)?;
                let len_bytes = row_bytes.get(offset..len_end)?;
                let len = u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;
                let value_end = len_end.checked_add(len)?;
                let value_bytes = row_bytes.get(len_end..value_end)?;

                let mut serialized = Vec::with_capacity(4 + len);
                serialized.extend_from_slice(len_bytes);
                serialized.extend_from_slice(value_bytes);
                values.push(serialized);
                offset = value_end;
            }
        }
    }

    Some(values)
}

fn collect_rows(reader: &CoreFileReader, schema: &Schema) -> Option<Vec<Vec<u8>>> {
    let mut all_rows = Vec::new();
    let row_groups = reader.read_all_row_groups().ok()?;

    for row_group in row_groups {
        let decoded_columns = row_group.decode_columns().ok()?;
        let row_count = row_group.row_count as usize;
        let mut col_offsets = vec![0usize; row_group.columns.len()];

        for row_idx in 0..row_count {
            let mut row_data = Vec::new();

            for (col_idx, field) in schema.fields.iter().enumerate() {
                let decoded_column = decoded_columns.get(col_idx)?;

                match field.field_type.fixed_size() {
                    Some(field_size) => {
                        let start = row_idx.checked_mul(field_size)?;
                        let end = start.checked_add(field_size)?;
                        let slice = decoded_column.get(start..end)?;
                        row_data.extend_from_slice(slice);
                    }
                    None => {
                        let offset = col_offsets[col_idx];
                        let len_end = offset.checked_add(4)?;
                        let len_bytes = decoded_column.get(offset..len_end)?;
                        let len = u32::from_le_bytes([
                            len_bytes[0],
                            len_bytes[1],
                            len_bytes[2],
                            len_bytes[3],
                        ]) as usize;
                        let value_start = len_end;
                        let value_end = value_start.checked_add(len)?;
                        let value = decoded_column.get(value_start..value_end)?;

                        row_data.extend_from_slice(len_bytes);
                        row_data.extend_from_slice(value);
                        col_offsets[col_idx] = value_end;
                    }
                }
            }

            all_rows.push(row_data);
        }
    }

    Some(all_rows)
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
                    let ptr = malloc(len) as *mut u8;
                    if ptr.is_null() {
                        return -1;
                    }

                    std::ptr::copy_nonoverlapping(buffer.as_ptr(), ptr, len);

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
        let slice = slice::from_raw_parts(data, size);
        let temp_path = temp_reader_path();

        if fs::write(&temp_path, slice).is_err() {
            eprintln!("qrd_reader_new_ffi: failed to write temp file");
            return std::ptr::null_mut();
        }

        let reader = match CoreFileReader::new(&temp_path) {
            Ok(reader) => reader,
            Err(_) => {
                eprintln!("qrd_reader_new_ffi: CoreFileReader::new failed");
                let _ = fs::remove_file(&temp_path);
                return std::ptr::null_mut();
            }
        };

        let schema = reader.schema().clone();
        let rows = match collect_rows(&reader, &schema) {
            Some(rows) => rows,
            None => {
                eprintln!("qrd_reader_new_ffi: collect_rows() failed");
                let _ = fs::remove_file(&temp_path);
                return std::ptr::null_mut();
            }
        };

        let _ = fs::remove_file(&temp_path);

        let ffi_reader = FFIReader {
            schema,
            rows,
            row_index: 0,
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

/// FFI function to get reader schema.
#[no_mangle]
pub extern "C" fn qrd_reader_schema_ffi(reader: *mut FFIReader) -> *mut FFISchema {
    if reader.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let reader_ref = &mut *reader;
        let schema = FFISchema {
            inner: reader_ref.schema.clone(),
        };
        Box::into_raw(Box::new(schema))
    }
}

/// FFI function to get reader row count.
#[no_mangle]
pub extern "C" fn qrd_reader_row_count_ffi(reader: *mut FFIReader) -> u64 {
    if reader.is_null() {
        return 0;
    }

    unsafe { (*reader).rows.len() as u64 }
}

/// FFI function to read next row.
#[no_mangle]
pub extern "C" fn qrd_reader_read_row_ffi(reader: *mut FFIReader) -> *mut FFIRow {
    if reader.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let reader_ref = &mut *reader;
        let row_bytes = match reader_ref.rows.get(reader_ref.row_index) {
            Some(row) => row,
            None => return std::ptr::null_mut(),
        };

        reader_ref.row_index += 1;

        let values = match split_row_bytes(row_bytes, &reader_ref.schema) {
            Some(values) => values,
            None => return std::ptr::null_mut(),
        };

        Box::into_raw(Box::new(FFIRow { values }))
    }
}

/// FFI function to create a new row.
#[no_mangle]
pub extern "C" fn qrd_row_new_ffi() -> *mut FFIRow {
    Box::into_raw(Box::new(FFIRow { values: Vec::new() }))
}

/// FFI function to free a row.
#[no_mangle]
pub extern "C" fn qrd_row_free_ffi(row: *mut FFIRow) {
    if !row.is_null() {
        unsafe {
            let _ = Box::from_raw(row);
        }
    }
}

/// FFI function to get field count from a row.
#[no_mangle]
pub extern "C" fn qrd_row_field_count_ffi(row: *const FFIRow) -> usize {
    if row.is_null() {
        return 0;
    }

    unsafe { (*row).values.len() }
}

/// FFI function to append raw bytes to a row.
#[no_mangle]
pub extern "C" fn qrd_row_add_bytes_ffi(row: *mut FFIRow, data: *const u8, size: usize) -> i32 {
    if row.is_null() {
        return -1;
    }

    unsafe {
        let row_ref = &mut *row;
        if size == 0 {
            row_ref.values.push(Vec::new());
            return 0;
        }

        if data.is_null() {
            return -1;
        }

        let slice = slice::from_raw_parts(data, size);
        row_ref.values.push(slice.to_vec());
        0
    }
}

/// FFI function to append an int64 value to a row.
#[no_mangle]
pub extern "C" fn qrd_row_add_int64_ffi(row: *mut FFIRow, value: i64) -> i32 {
    let bytes = value.to_le_bytes();
    qrd_row_add_bytes_ffi(row, bytes.as_ptr(), bytes.len())
}

/// FFI function to append a float64 value to a row.
#[no_mangle]
pub extern "C" fn qrd_row_add_float64_ffi(row: *mut FFIRow, value: f64) -> i32 {
    let bytes = value.to_le_bytes();
    qrd_row_add_bytes_ffi(row, bytes.as_ptr(), bytes.len())
}

/// FFI function to append a string value to a row.
#[no_mangle]
pub extern "C" fn qrd_row_add_string_ffi(row: *mut FFIRow, value: *const c_char) -> i32 {
    if row.is_null() {
        return -1;
    }

    unsafe {
        if value.is_null() {
            return qrd_row_add_bytes_ffi(row, std::ptr::null(), 0);
        }

        let bytes = CStr::from_ptr(value).to_bytes();
        let mut serialized = Vec::with_capacity(4 + bytes.len());
        serialized.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        serialized.extend_from_slice(bytes);
        qrd_row_add_bytes_ffi(row, serialized.as_ptr(), serialized.len())
    }
}
