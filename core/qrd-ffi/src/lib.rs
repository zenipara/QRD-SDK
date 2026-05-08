//! QRD FFI - C Foreign Function Interface

use qrd_core::prelude::*;

/// FFI wrapper for schema
pub struct FFISchema {
    inner: Schema,
}

/// FFI function to create schema (stub)
#[no_mangle]
pub extern "C" fn qrd_schema_new() -> *mut FFISchema {
    Box::into_raw(Box::new(FFISchema {
        inner: Schema {
            fields: vec![],
            schema_id: 0,
        },
    }))
}

/// FFI function to free schema
#[no_mangle]
pub extern "C" fn qrd_schema_free(schema: *mut FFISchema) {
    if !schema.is_null() {
        unsafe {
            let _ = Box::from_raw(schema);
        }
    }
}
