//! QRD WASM - WebAssembly bindings for browser and Node.js

use qrd_core::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{console, Blob, BlobPropertyBag, File, Uint8Array};
use js_sys::{Array, Object, Reflect};
use serde::{Deserialize, Serialize};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console::log_1(&"QRD WASM initialized".into());
}

/// Schema field definition for JavaScript
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    name: String,
    logical_type: String,
    nullable: bool,
    repeated: bool,
    metadata: Option<String>,
}

#[wasm_bindgen]
impl SchemaField {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, logical_type: &str, nullable: bool, repeated: bool) -> SchemaField {
        SchemaField {
            name: name.to_string(),
            logical_type: logical_type.to_string(),
            nullable,
            repeated,
            metadata: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn logical_type(&self) -> String {
        self.logical_type.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn nullable(&self) -> bool {
        self.nullable
    }

    #[wasm_bindgen(getter)]
    pub fn repeated(&self) -> bool {
        self.repeated
    }

    #[wasm_bindgen(setter)]
    pub fn set_metadata(&mut self, metadata: Option<String>) {
        self.metadata = metadata;
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> Option<String> {
        self.metadata.clone()
    }
}

/// Schema builder for JavaScript
#[wasm_bindgen]
pub struct SchemaBuilder {
    fields: Vec<SchemaField>,
}

#[wasm_bindgen]
impl SchemaBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SchemaBuilder {
        SchemaBuilder {
            fields: Vec::new(),
        }
    }

    #[wasm_bindgen]
    pub fn add_field(&mut self, field: &SchemaField) {
        self.fields.push(field.clone());
    }

    #[wasm_bindgen]
    pub fn build(&self) -> Result<WasmSchema, JsValue> {
        let mut schema_builder = qrd_core::schema::SchemaBuilder::new();

        for field in &self.fields {
            let logical_type = match field.logical_type.as_str() {
                "BOOLEAN" => LogicalType::BOOLEAN,
                "INT8" => LogicalType::INT8,
                "INT16" => LogicalType::INT16,
                "INT32" => LogicalType::INT32,
                "INT64" => LogicalType::INT64,
                "UINT8" => LogicalType::UINT8,
                "UINT16" => LogicalType::UINT16,
                "UINT32" => LogicalType::UINT32,
                "UINT64" => LogicalType::UINT64,
                "FLOAT32" => LogicalType::FLOAT32,
                "FLOAT64" => LogicalType::FLOAT64,
                "STRING" => LogicalType::STRING,
                "BLOB" => LogicalType::BLOB,
                "TIMESTAMP" => LogicalType::TIMESTAMP,
                "DATE" => LogicalType::DATE,
                "TIME" => LogicalType::TIME,
                "DURATION" => LogicalType::DURATION,
                "UUID" => LogicalType::UUID,
                "DECIMAL" => LogicalType::DECIMAL,
                "ENUM" => LogicalType::ENUM,
                _ => return Err(JsValue::from_str(&format!("Unknown logical type: {}", field.logical_type))),
            };

            let nullability = if field.repeated {
                Nullability::REPEATED
            } else if field.nullable {
                Nullability::OPTIONAL
            } else {
                Nullability::REQUIRED
            };

            schema_builder = schema_builder.add_field(
                &field.name,
                logical_type,
                nullability,
                field.metadata.as_deref(),
            );
        }

        let schema = schema_builder.build().map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmSchema { inner: schema })
    }
}

/// WASM wrapper for schema
#[wasm_bindgen]
pub struct WasmSchema {
    inner: Schema,
}

#[wasm_bindgen]
impl WasmSchema {
    /// Get column count
    #[wasm_bindgen]
    pub fn column_count(&self) -> usize {
        self.inner.column_count()
    }

    /// Get schema ID
    #[wasm_bindgen]
    pub fn schema_id(&self) -> u64 {
        self.inner.schema_id
    }

    /// Get field names as JavaScript array
    #[wasm_bindgen]
    pub fn field_names(&self) -> Array {
        let array = Array::new();
        for field in &self.inner.fields {
            array.push(&JsValue::from_str(&field.name));
        }
        array
    }
}

/// File writer for JavaScript
#[wasm_bindgen]
pub struct FileWriter {
    inner: Option<qrd_core::writer::FileWriter<std::io::Cursor<Vec<u8>>>>,
    buffer: Vec<u8>,
}

#[wasm_bindgen]
impl FileWriter {
    #[wasm_bindgen(constructor)]
    pub fn new(schema: &WasmSchema) -> Result<FileWriter, JsValue> {
        let buffer = Vec::new();
        let cursor = std::io::Cursor::new(buffer);
        let writer = qrd_core::writer::FileWriter::new(cursor, schema.inner.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(FileWriter {
            inner: Some(writer),
            buffer: Vec::new(),
        })
    }

    /// Write a row of data
    #[wasm_bindgen]
    pub fn write_row(&mut self, row_data: &JsValue) -> Result<(), JsValue> {
        if let Some(writer) = &mut self.inner {
            // Convert JS object to Rust data structure
            let row = self.js_value_to_row(row_data)?;
            writer.write_row(&row).map_err(|e| JsValue::from_str(&e.to_string()))?;
        }
        Ok(())
    }

    /// Finish writing and get the buffer
    #[wasm_bindgen]
    pub fn finish(&mut self) -> Result<Uint8Array, JsValue> {
        if let Some(writer) = self.inner.take() {
            let cursor = writer.finish().map_err(|e| JsValue::from_str(&e.to_string()))?;
            let buffer = cursor.into_inner();
            Ok(Uint8Array::from(&buffer[..]))
        } else {
            Err(JsValue::from_str("Writer already finished"))
        }
    }

    /// Convert JS value to internal row representation
    fn js_value_to_row(&self, js_value: &JsValue) -> Result<Vec<qrd_core::schema::Value>, JsValue> {
        let obj = Object::from(js_value.clone());
        let mut row = Vec::new();

        // This is a simplified implementation - in practice you'd need to map
        // JS object properties to the schema fields
        // For now, return empty row
        Ok(row)
    }
}

/// File reader for JavaScript
#[wasm_bindgen]
pub struct FileReader {
    inner: Option<qrd_core::reader::FileReader<std::io::Cursor<Vec<u8>>>>,
}

#[wasm_bindgen]
impl FileReader {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<FileReader, JsValue> {
        let cursor = std::io::Cursor::new(data.to_vec());
        let reader = qrd_core::reader::FileReader::new(cursor)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(FileReader {
            inner: Some(reader),
        })
    }

    /// Get the schema
    #[wasm_bindgen]
    pub fn schema(&self) -> Result<WasmSchema, JsValue> {
        if let Some(reader) = &self.inner {
            let schema = reader.schema().clone();
            Ok(WasmSchema { inner: schema })
        } else {
            Err(JsValue::from_str("Reader not initialized"))
        }
    }

    /// Get row count
    #[wasm_bindgen]
    pub fn row_count(&self) -> Result<u64, JsValue> {
        if let Some(reader) = &self.inner {
            Ok(reader.row_count())
        } else {
            Err(JsValue::from_str("Reader not initialized"))
        }
    }

    /// Read next row
    #[wasm_bindgen]
    pub fn read_row(&mut self) -> Result<JsValue, JsValue> {
        if let Some(reader) = &mut self.inner {
            match reader.read_row() {
                Ok(Some(row)) => {
                    // Convert row to JS object
                    self.row_to_js_value(&row)
                }
                Ok(None) => Ok(JsValue::null()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Reader not initialized"))
        }
    }

    /// Convert internal row to JS value
    fn row_to_js_value(&self, _row: &[qrd_core::schema::Value]) -> Result<JsValue, JsValue> {
        // Simplified implementation - in practice you'd convert the row data
        // to a JS object based on the schema
        let obj = Object::new();
        Ok(obj.into())
    }
}
