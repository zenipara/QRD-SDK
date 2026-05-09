//! QRD WASM — WebAssembly bindings for browser and Node.js

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
        QrdSchemaBuilder {
            inner: SchemaBuilder::new(),
        }
    }

    pub fn add_field(&mut self, name: &str, field_type: &str) -> Result<(), JsValue> {
        let ft = parse_field_type(field_type).map_err(|e| JsValue::from_str(&e))?;
        self.inner
            .add_field(name, ft, Nullability::Required)
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
    writer: Option<StreamingWriter<Cursor<Vec<u8>>>>,
}

#[wasm_bindgen]
impl QrdMemWriter {
    #[wasm_bindgen(constructor)]
    pub fn new(schema: &QrdSchema) -> Result<QrdMemWriter, JsValue> {
        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let writer = StreamingWriter::new(cursor, schema.inner.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(QrdMemWriter {
            writer: Some(writer),
        })
    }

    pub fn write_row(&mut self, columns: Vec<js_sys::Uint8Array>) -> Result<(), JsValue> {
        let writer = self
            .writer
            .as_mut()
            .ok_or("Writer finished")?;
        let row: Vec<Vec<u8>> = columns.iter().map(|a| a.to_vec()).collect();
        writer.write_row(row).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn finish(mut self) -> Result<js_sys::Uint8Array, JsValue> {
        let writer = self
            .writer
            .take()
            .ok_or("Already finished")?;
        let cursor = writer.finish_into_inner()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let bytes = cursor.into_inner();
        Ok(js_sys::Uint8Array::from(bytes.as_slice()))
    }
}

fn parse_field_type(s: &str) -> Result<FieldType, String> {
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
        "DATE" => Ok(FieldType::Date),
        "TIME" => Ok(FieldType::Time),
        "UUID" => Ok(FieldType::Uuid),
        "DECIMAL" => Ok(FieldType::Decimal),
        _ => Err(format!("Unknown field type: {}", s)),
    }
}

impl WasmReader {
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
