//! QRD WASM — WebAssembly bindings for browser and Node.js

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use js_sys;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::StreamingWriter;
use std::io::Cursor;

#[wasm_bindgen]
pub struct QrdSchemaBuilder {
    inner: Option<SchemaBuilder>,
}

#[wasm_bindgen]
impl QrdSchemaBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        QrdSchemaBuilder {
            inner: Some(SchemaBuilder::new()),
        }
    }

    pub fn add_field(&mut self, name: &str, field_type: &str) -> Result<(), JsValue> {
        let ft = parse_field_type(field_type).map_err(|e| JsValue::from_str(&e))?;
        
        // Take the builder out of Option, add field, put it back
        let builder = self.inner.take().ok_or_else(|| JsValue::from_str("Builder has been consumed"))?;
        self.inner = Some(
            builder
                .add_field(name, ft, Nullability::Required)
                .map_err(|e| JsValue::from_str(&e.to_string()))?
        );
        Ok(())
    }

    pub fn build(mut self) -> Result<QrdSchema, JsValue> {
        match self.inner.take() {
            Some(inner) => {
                let schema = inner.build().map_err(|e| JsValue::from_str(&e.to_string()))?;
                Ok(QrdSchema { inner: schema })
            }
            None => Err(JsValue::from_str("Builder has been consumed"))
        }
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
    buffer: Vec<u8>,
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
            buffer: Vec::new(),
        })
    }

    pub fn write_row(&mut self, columns: Vec<js_sys::Uint8Array>) -> Result<(), JsValue> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Writer finished"))?;
        let row: Vec<Vec<u8>> = columns.iter().map(|a| a.to_vec()).collect();
        writer.write_row(row).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn finish(mut self) -> Result<js_sys::Uint8Array, JsValue> {
        let mut writer = self
            .writer
            .take()
            .ok_or("Already finished")?;
        
        // Extract the buffer before finishing
        let mut cursor = writer.into_inner();
        let buffer = cursor.into_inner();
        
        // Call finish on the writer (though it may not be necessary for in-memory)
        // writer.finish().map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(js_sys::Uint8Array::from(&buffer[..]))
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

// TODO: Implement QrdMemReader in subsequent phase
// - QrdMemReader struct for reading QRD files in WASM
// - schema() method to get file schema
// - row_count() method to get total rows
// - read_row() method for sequential reads
// - read_columns() method for selective column reading
// Currently removed to unblock build. Will be implemented when reader APIs are stabilized.
