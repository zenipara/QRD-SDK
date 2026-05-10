//! QRD C FFI — Stable C interface for all language bindings
//!
//! Memory ownership rules:
//! - Functions returning *mut T transfer ownership to caller
//! - Caller MUST call the corresponding _free function
//! - Input *const T are borrowed — caller retains ownership
//! - Returning i32 error codes: 0 = success, negative = error

use qrd_core::prelude::*;
use qrd_core::reader::FileReader as CoreFileReader;
use std::cell::RefCell;
use std::ffi::c_void;
use std::ffi::CStr;
use std::io::{self, Write};
use std::os::raw::{c_char, c_int};
use std::rc::Rc;
use std::slice;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use std::sync::Mutex as StdMutex;

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
}

static TEMP_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
struct SharedVecWriter {
    /// Thread-safe buffer using Arc<Mutex<>> instead of Rc<RefCell<>>
    /// This allows safe sharing across thread boundaries if needed
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl SharedVecWriter {
    fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn bytes(&self) -> Vec<u8> {
        self.buffer.lock().unwrap().clone()
    }
}

impl Write for SharedVecWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.lock().unwrap().extend_from_slice(buf);
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
    schema: Schema,
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
    let row_groups = match reader.read_all_row_groups() {
        Ok(rg) => rg,
        Err(e) => {
            eprintln!("collect_rows: read_all_row_groups() error: {:?}", e);
            return None;
        }
    };

    eprintln!("collect_rows: found {} row_groups", row_groups.len());

    for (rg_idx, row_group) in row_groups.into_iter().enumerate() {
        eprintln!("collect_rows: processing row_group {}: row_count={}, columns={}", rg_idx, row_group.row_count, row_group.columns.len());

        let decoded_columns = match row_group.decode_columns() {
            Ok(dc) => dc,
            Err(e) => {
                eprintln!("collect_rows: row_group.decode_columns() failed: {:?}", e);
                return None;
            }
        };

        let row_count = row_group.row_count as usize;
        let mut col_offsets = vec![0usize; row_group.columns.len()];

        for (col_idx, col_buf) in decoded_columns.iter().enumerate() {
            let encoded_len = row_group.columns.get(col_idx).map(|c| c.encoded_data.len()).unwrap_or(0);
            let encoding_id = row_group.columns.get(col_idx).map(|c| c.encoding.to_id()).unwrap_or(255);
            eprintln!("collect_rows: decoded column {} length={} encoding={} encoded_len={}", col_idx, col_buf.len(), encoding_id, encoded_len);
            if col_buf.len() > 0 {
                let preview = &col_buf[..std::cmp::min(8, col_buf.len())];
                eprintln!("collect_rows: col {} preview={:02x?}", col_idx, preview);
            }
        }

        for row_idx in 0..row_count {
            let mut row_data = Vec::new();

            for (col_idx, field) in schema.fields.iter().enumerate() {
                let decoded_column = match decoded_columns.get(col_idx) {
                    Some(dc) => dc,
                    None => {
                        eprintln!("collect_rows: missing decoded column {}", col_idx);
                        return None;
                    }
                };

                match field.field_type.fixed_size() {
                    Some(field_size) => {
                        let start = match row_idx.checked_mul(field_size) {
                            Some(s) => s,
                            None => {
                                eprintln!("collect_rows: overflow computing start for fixed field");
                                return None;
                            }
                        };
                        let end = match start.checked_add(field_size) {
                            Some(e) => e,
                            None => {
                                eprintln!("collect_rows: overflow computing end for fixed field");
                                return None;
                            }
                        };
                        let slice = match decoded_column.get(start..end) {
                            Some(s) => s,
                            None => {
                                eprintln!("collect_rows: fixed field slice out of bounds: start={} end={} col_len={}", start, end, decoded_column.len());
                                return None;
                            }
                        };
                        row_data.extend_from_slice(slice);
                    }
                    None => {
                        let offset = col_offsets[col_idx];
                        let len_end = match offset.checked_add(4) {
                            Some(le) => le,
                            None => {
                                eprintln!("collect_rows: overflow computing len_end");
                                return None;
                            }
                        };
                        let len_bytes = match decoded_column.get(offset..len_end) {
                            Some(lb) => lb,
                            None => {
                                eprintln!("collect_rows: varlen len_bytes out of bounds: offset={} len_end={} col_len={}", offset, len_end, decoded_column.len());
                                return None;
                            }
                        };
                        let len = u32::from_le_bytes([
                            len_bytes[0],
                            len_bytes[1],
                            len_bytes[2],
                            len_bytes[3],
                        ]) as usize;
                        let value_start = len_end;
                        let value_end = match value_start.checked_add(len) {
                            Some(ve) => ve,
                            None => {
                                eprintln!("collect_rows: overflow computing value_end");
                                return None;
                            }
                        };
                        let value = match decoded_column.get(value_start..value_end) {
                            Some(v) => v,
                            None => {
                                eprintln!("collect_rows: varlen value out of bounds: start={} end={} col_len={}", value_start, value_end, decoded_column.len());
                                return None;
                            }
                        };

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

#[cfg(test)]
mod tests {
    use super::*;
    use qrd_core::writer::StreamingWriter;

    #[test]
    fn test_collect_rows_roundtrip() {
        let mut builder = SchemaBuilder::new();
        builder = builder.add_field("id", FieldType::Int64, Nullability::Required).unwrap();
        builder = builder.add_field("name", FieldType::String, Nullability::Required).unwrap();
        let schema = builder.build().unwrap();

        let buffer = SharedVecWriter::new();
        let mut writer = StreamingWriter::new(buffer.clone(), schema.clone()).unwrap();

        for i in 0..3 {
            let mut row: Vec<Vec<u8>> = Vec::new();
            row.push((i as i64).to_le_bytes().to_vec());
            let name = format!("name{}", i);
            let mut s = Vec::new();
            s.extend_from_slice(&(name.len() as u32).to_le_bytes());
            s.extend_from_slice(name.as_bytes());
            row.push(s);
            writer.write_row(row).unwrap();
        }

        writer.finish().unwrap();
        let bytes = buffer.bytes();
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.as_file_mut().write_all(&bytes).unwrap();
        let reader = CoreFileReader::new(tmp.path()).unwrap();
        let collected = collect_rows(&reader, &schema).expect("collect_rows failed");
        assert_eq!(collected.len(), 3);
    }

    #[test]
    fn test_writer_zero_length_finish() {
        // Test that writer with no rows finishes without calling malloc(0)
        let mut builder = SchemaBuilder::new();
        builder = builder.add_field("id", FieldType::Int64, Nullability::Required).unwrap();
        let schema = builder.build().unwrap();

        let buffer = SharedVecWriter::new();
        let writer = StreamingWriter::new(buffer.clone(), schema).unwrap();
        
        // Finish without writing any rows
        writer.finish().unwrap();
        let bytes = buffer.bytes();
        
        // The finish() call should produce a valid QRD file (with headers/footer)
        // but in the FFI finish, if somehow the final buffer is empty, it should
        // safely return NULL/0 without calling malloc(0).
        // This test primarily validates the structural soundness of zero-row output.
        assert!(bytes.len() > 0, "Expected QRD file with headers/footer even for zero rows");
    }
}

// ============================================================================
// FFI HELPER TYPES & ERROR HANDLING
// ============================================================================

/// Simple wrapper for SchemaBuilder to use in FFI
#[repr(C)]
pub struct QrdSchemaBuilder(Option<SchemaBuilder>);

/// Thread-safe global error storage for last FFI error
thread_local! {
    static LAST_ERROR: StdMutex<String> = StdMutex::new(String::new());
}

fn set_last_error(msg: String) {
    LAST_ERROR.with(|e| {
        if let Ok(mut err) = e.lock() {
            *err = msg;
        }
    });
}

fn field_type_from_int(value: i32) -> std::result::Result<FieldType, String> {
    match value {
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
        14 => Ok(FieldType::Duration),
        15 => Ok(FieldType::String),
        16 => Ok(FieldType::Enum),
        17 => Ok(FieldType::Uuid),
        18 => Ok(FieldType::Blob),
        19 => Ok(FieldType::Decimal),
        n => Err(format!("Unknown field type: {}", n)),
    }
}

fn nullability_from_int(value: i32) -> std::result::Result<Nullability, String> {
    match value {
        0 => Ok(Nullability::Required),
        1 => Ok(Nullability::Optional),
        2 => Ok(Nullability::Repeated),
        n => Err(format!("Unknown nullability: {}", n)),
    }
}

// ============================================================================
// FIELD TYPE CONVERSION (LEGACY)
// ============================================================================

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
pub extern "C" fn qrd_schema_builder_new() -> *mut QrdSchemaBuilder {
    Box::into_raw(Box::new(QrdSchemaBuilder(Some(SchemaBuilder::new()))))
}

#[no_mangle]
pub extern "C" fn qrd_schema_builder_free(ptr: *mut QrdSchemaBuilder) {
    if !ptr.is_null() { 
        unsafe { drop(Box::from_raw(ptr)); } 
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
    if builder.is_null() {
        set_last_error("Builder pointer is null".to_string());
        return -1;
    }

    let name_str = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    
    let ft = match field_type_from_int(field_type) {
        Ok(ft) => ft,
        Err(e) => { set_last_error(e); return -1; }
    };

    let null = match nullability_from_int(nullability) {
        Ok(n) => n,
        Err(e) => { set_last_error(e); return -1; }
    };

    unsafe {
        let builder_ref = &mut *builder;
        
        // Extract the builder from Option, or create a new one if none exists
        let old_builder = builder_ref.0.take().unwrap_or_else(SchemaBuilder::new);
        
        // Add the new field
        match old_builder.add_field(name_str, ft, null) {
            Ok(new_builder) => {
                builder_ref.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(format!("add_field failed: {}", e));
                -1
            }
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
            schema: (*schema).inner.clone(),
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
            // Normalize variable-length optional empty values to length-prefix zero
            let mut normalized: Vec<Vec<u8>> = Vec::with_capacity(row_ref.values.len());
            for (i, val) in row_ref.values.iter().enumerate() {
                if val.is_empty() {
                    if let Some(field) = writer_ref.schema.fields.get(i) {
                        if field.nullability == qrd_core::schema::Nullability::Optional && field.field_type.fixed_size().is_none() {
                            normalized.push(vec![0, 0, 0, 0]);
                            continue;
                        }
                    }
                }
                normalized.push(val.clone());
            }

            match w.write_row(normalized) {
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
                    
                    if len == 0 {
                        // For zero-length output, return NULL without calling malloc
                        *data = std::ptr::null_mut();
                        *size = 0;
                        return 0;
                    }
                    
                    let ptr = malloc(len) as *mut u8;
                    if ptr.is_null() {
                        return -1;
                    }

                    std::ptr::copy_nonoverlapping(buffer.as_ptr(), ptr, len);

                    *data = ptr;
                    *size = len;
                    return 0;
                }
                Err(e) => { 
                    set_last_error(e.to_string()); 
                    return -1;
                }
            }
        } else {
            set_last_error("Builder has been consumed".to_string());
            return -1;
        }
    }
}

#[no_mangle]
pub extern "C" fn qrd_reader_new_ffi(data: *const u8, size: usize) -> *mut FFIReader {
    if data.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let slice = slice::from_raw_parts(data, size);
        eprintln!("qrd_reader_new_ffi: incoming size={}", size);
        if size > 0 {
            let preview_len = std::cmp::min(8, size);
            eprintln!("qrd_reader_new_ffi: preview={:02x?}", &slice[..preview_len]);
        }
        // Use NamedTempFile for safer temporary file handling
        let mut tmp = match NamedTempFile::new() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("qrd_reader_new_ffi: failed to create temp file: {:?}", e);
                return std::ptr::null_mut();
            }
        };

        if tmp.as_file_mut().write_all(slice).is_err() {
            eprintln!("qrd_reader_new_ffi: failed to write temp file");
            return std::ptr::null_mut();
        }

        let reader = match CoreFileReader::new(tmp.path()) {
            Ok(reader) => reader,
            Err(_) => {
                eprintln!("qrd_reader_new_ffi: CoreFileReader::new failed");
                return std::ptr::null_mut();
            }
        };

        let schema = reader.schema().clone();
        let rows = match collect_rows(&reader, &schema) {
            Some(rows) => rows,
            None => {
                eprintln!("qrd_reader_new_ffi: collect_rows() failed");
                return std::ptr::null_mut();
            }
        };

        // `tmp` will be removed when dropped after this function returns

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
