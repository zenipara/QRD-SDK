//! QRD WASM — WebAssembly bindings for browser and Node.js

use js_sys::{Array, Uint8Array};
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::StreamingWriter;
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

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
        self.add_field_with_nullability(name, field_type, "REQUIRED")
    }

    pub fn add_field_with_nullability(
        &mut self,
        name: &str,
        field_type: &str,
        nullability: &str,
    ) -> Result<(), JsValue> {
        let ft = parse_field_type(field_type).map_err(|e| JsValue::from_str(&e))?;
        let nn = parse_nullability(nullability).map_err(|e| JsValue::from_str(&e))?;

        let builder = self
            .inner
            .take()
            .ok_or_else(|| JsValue::from_str("Builder has been consumed"))?;
        self.inner = Some(
            builder
                .add_field(name, ft, nn)
                .map_err(|e| JsValue::from_str(&e.to_string()))?,
        );
        Ok(())
    }

    pub fn build(mut self) -> Result<QrdSchema, JsValue> {
        match self.inner.take() {
            Some(inner) => {
                let schema = inner
                    .build()
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                Ok(QrdSchema { inner: schema })
            }
            None => Err(JsValue::from_str("Builder has been consumed")),
        }
    }
}

#[wasm_bindgen]
pub struct QrdSchema {
    inner: qrd_core::schema::Schema,
}

#[wasm_bindgen]
impl QrdSchema {
    #[wasm_bindgen(getter)]
    pub fn schema_id(&self) -> u32 {
        self.inner.schema_id
    }

    pub fn field_count(&self) -> usize {
        self.inner.fields.len()
    }

    pub fn field_name(&self, index: usize) -> Result<String, JsValue> {
        self.inner
            .fields
            .get(index)
            .map(|field| field.name.clone())
            .ok_or_else(|| JsValue::from_str("Field index out of bounds"))
    }

    pub fn field_type(&self, index: usize) -> Result<String, JsValue> {
        self.inner
            .fields
            .get(index)
            .map(|field| field.field_type.to_string())
            .ok_or_else(|| JsValue::from_str("Field index out of bounds"))
    }

    pub fn nullability(&self, index: usize) -> Result<String, JsValue> {
        self.inner
            .fields
            .get(index)
            .map(|field| field.nullability.to_string())
            .ok_or_else(|| JsValue::from_str("Field index out of bounds"))
    }
}

/// In-memory writer — returns bytes when finished
#[wasm_bindgen]
pub struct QrdMemWriter {
    schema: qrd_core::schema::Schema,
    rows: Vec<Vec<Vec<u8>>>,
}

#[wasm_bindgen]
impl QrdMemWriter {
    #[wasm_bindgen(constructor)]
    pub fn new(schema: &QrdSchema) -> Result<QrdMemWriter, JsValue> {
        Ok(QrdMemWriter {
            schema: schema.inner.clone(),
            rows: Vec::new(),
        })
    }

    pub fn write_row(&mut self, columns: Vec<Uint8Array>) -> Result<(), JsValue> {
        let row: Vec<Vec<u8>> = columns.iter().map(|a| a.to_vec()).collect();
        self.rows.push(row);
        Ok(())
    }

    pub fn finish(self) -> Result<Uint8Array, JsValue> {
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);
        let mut writer = StreamingWriter::new(cursor, self.schema.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        for row in self.rows {
            writer
                .write_row(row)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }

        writer.finish().map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Uint8Array::from(buffer.as_slice()))
    }
}

#[wasm_bindgen]
pub struct QrdMemReader {
    inner: qrd_core::reader::FileReader,
}

#[wasm_bindgen]
impl QrdMemReader {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>) -> Result<QrdMemReader, JsValue> {
        let reader = qrd_core::reader::FileReader::from_bytes(data)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(QrdMemReader { inner: reader })
    }

    pub fn schema(&self) -> QrdSchema {
        QrdSchema {
            inner: self.inner.schema().clone(),
        }
    }

    pub fn row_count(&self) -> u32 {
        self.inner.row_count()
    }

    pub fn read_row(&self, row_index: u32) -> Result<Uint8Array, JsValue> {
        let rows = self
            .inner
            .rows()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let row = rows
            .get(row_index as usize)
            .ok_or_else(|| JsValue::from_str("Row index out of bounds"))?;

        Ok(Uint8Array::from(row.as_slice()))
    }

    pub fn read_columns(&self, column_indices: Vec<usize>) -> Result<Array, JsValue> {
        let columns = self
            .inner
            .read_columns(&column_indices)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let array = Array::new();
        for column in columns {
            array.push(&Uint8Array::from(column.as_slice()).into());
        }

        Ok(array)
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

fn parse_nullability(s: &str) -> Result<Nullability, String> {
    match s.to_uppercase().as_str() {
        "REQUIRED" => Ok(Nullability::Required),
        "OPTIONAL" => Ok(Nullability::Optional),
        "REPEATED" => Ok(Nullability::Repeated),
        _ => Err(format!("Unknown nullability: {}", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
    use qrd_core::writer::StreamingWriter;
    use std::io::Cursor;
    use wasm_bindgen::JsCast;

    #[test]
    #[ignore]  // WASM tests cannot run on non-WASM targets
    fn test_qrd_mem_reader_roundtrip() {
        let schema = SchemaBuilder::new()
            .add_field("id", FieldType::Int64, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = StreamingWriter::new(cursor, schema.clone()).unwrap();
            writer
                .write_row(vec![vec![1u8, 0, 0, 0, 0, 0, 0, 0]])
                .unwrap();
            writer.finish().unwrap();
        }

        let reader = QrdMemReader::new(buffer).unwrap();
        assert_eq!(reader.row_count(), 1);
        assert_eq!(reader.schema().field_count(), 1);
        assert_eq!(reader.schema().schema_id(), schema.schema_id);

        let row = reader.read_row(0).unwrap();
        assert_eq!(row.length(), 8);

        let columns = reader.read_columns(vec![0]).unwrap();
        assert_eq!(columns.length(), 1);
        let first_column = columns.get(0);
        let first_column = first_column.unchecked_into::<Uint8Array>();
        assert_eq!(first_column.length(), 8);
    }

    #[test]
    fn test_qrd_schema_builder_nullability_support() {
        let mut builder = QrdSchemaBuilder::new();
        builder
            .add_field_with_nullability("id", "INT64", "REQUIRED")
            .unwrap();
        builder
            .add_field_with_nullability("name", "STRING", "OPTIONAL")
            .unwrap();
        builder
            .add_field_with_nullability("tags", "STRING", "REPEATED")
            .unwrap();
        let schema = builder.build().unwrap();

        assert_eq!(schema.field_count(), 3);
        assert_eq!(schema.field_name(0).unwrap(), "id");
        assert_eq!(schema.field_type(1).unwrap(), "STRING");
        assert_eq!(schema.nullability(2).unwrap(), "REPEATED");
    }
}
