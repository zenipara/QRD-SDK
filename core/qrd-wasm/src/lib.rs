//! QRD WASM - WebAssembly implementation

use qrd_core::prelude::*;

/// WASM wrapper for schema (stub)
pub struct WasmSchema {
    inner: Schema,
}

impl WasmSchema {
    /// Create new schema
    pub fn new() -> Self {
        WasmSchema {
            inner: Schema {
                fields: vec![],
                schema_id: 0,
            },
        }
    }

    /// Get column count
    pub fn column_count(&self) -> usize {
        self.inner.column_count()
    }
}

impl Default for WasmSchema {
    fn default() -> Self {
        Self::new()
    }
}
