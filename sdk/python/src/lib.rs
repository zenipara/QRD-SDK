//! Python bindings for QRD SDK using PyO3

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyList;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::FileWriter;
use qrd_core::reader::FileReader;

fn qrd_error_to_py(e: qrd_core::error::Error) -> PyErr {
    PyValueError::new_err(e.to_string())
}

#[pyclass(name = "SchemaBuilder")]
struct PySchemaBuilder {
    inner: Option<SchemaBuilder>,
}

#[pymethods]
impl PySchemaBuilder {
    #[new]
    fn new() -> Self {
        PySchemaBuilder {
            inner: Some(SchemaBuilder::new()),
        }
    }

    fn add_field(&mut self, name: &str, field_type: &str, required: bool) -> PyResult<()> {
        let ft = parse_field_type(field_type)?;
        let null = if required {
            Nullability::Required
        } else {
            Nullability::Optional
        };
        
        match self.inner.take() {
            Some(inner) => {
                match inner.add_field(name, ft, null) {
                    Ok(new_inner) => {
                        self.inner = Some(new_inner);
                        Ok(())
                    }
                    Err(e) => Err(qrd_error_to_py(e))
                }
            }
            None => Err(PyValueError::new_err("Builder has been consumed"))
        }
    }

    fn build(&mut self) -> PyResult<PySchema> {
        match self.inner.take() {
            Some(inner) => {
                let schema = inner.build().map_err(qrd_error_to_py)?;
                Ok(PySchema { inner: schema })
            }
            None => Err(PyValueError::new_err("Builder has been consumed"))
        }
    }
}

#[pyclass(name = "Schema")]
#[derive(Clone)]
struct PySchema {
    inner: qrd_core::schema::Schema,
}

#[pymethods]
impl PySchema {
    fn field_count(&self) -> usize {
        self.inner.fields.len()
    }

    fn __repr__(&self) -> String {
        format!("Schema(fields={})", self.inner.fields.len())
    }
}

#[pyclass(name = "Writer")]
struct PyWriter {
    inner: Option<FileWriter>,
}

#[pymethods]
impl PyWriter {
    #[new]
    fn new(path: &str, schema: &PySchema) -> PyResult<Self> {
        let writer = FileWriter::new(path, schema.inner.clone()).map_err(qrd_error_to_py)?;
        Ok(PyWriter {
            inner: Some(writer),
        })
    }

    /// Write a row. columns is list of bytes objects.
    fn write_row(&mut self, columns: &Bound<'_, PyList>) -> PyResult<()> {
        let writer = self
            .inner
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Writer already finished"))?;
        
        let row: Vec<Vec<u8>> = columns
            .iter()
            .map(|item| {
                let bytes: &[u8] = item.extract()?;
                Ok(bytes.to_vec())
            })
            .collect::<PyResult<Vec<_>>>()?;
        
        writer.write_row(row).map_err(qrd_error_to_py)
    }

    fn finish(&mut self) -> PyResult<()> {
        let writer = self
            .inner
            .take()
            .ok_or_else(|| PyValueError::new_err("Already finished"))?;
        writer.finish().map_err(qrd_error_to_py)
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __exit__(
        &mut self,
        _exc_type: PyObject,
        _exc_val: PyObject,
        _exc_tb: PyObject,
    ) -> bool {
        if self.inner.is_some() {
            let _ = self.finish();
        }
        false
    }
}

#[pyclass(name = "Reader")]
struct PyReader {
    inner: FileReader,
}

#[pymethods]
impl PyReader {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let reader = FileReader::new(path).map_err(qrd_error_to_py)?;
        Ok(PyReader { inner: reader })
    }

    fn row_count(&self) -> u32 {
        self.inner.row_count()
    }

    fn schema(&self) -> PySchema {
        PySchema {
            inner: self.inner.schema().clone(),
        }
    }
}

fn parse_field_type(s: &str) -> PyResult<FieldType> {
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
        _ => Err(PyValueError::new_err(format!(
            "Unknown field type: {}",
            s
        ))),
    }
}

#[pymodule]
fn qrd(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySchemaBuilder>()?;
    m.add_class::<PySchema>()?;
    m.add_class::<PyWriter>()?;
    m.add_class::<PyReader>()?;
    Ok(())
}