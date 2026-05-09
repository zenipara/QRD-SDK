//! Python bindings for QRD SDK

use pyo3::prelude::*;
use qrd_core::prelude::*;

/// Python module for QRD columnar format
#[pymodule]
fn qrd(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PySchemaBuilder>()?;
    m.add_class::<PyFileWriter>()?;
    m.add_class::<PyFileReader>()?;
    m.add_class::<PyFieldType>()?;
    m.add_class::<PyNullability>()?;
    Ok(())
}

/// Schema builder for Python
#[pyclass]
struct PySchemaBuilder {
    builder: SchemaBuilder,
}

#[pymethods]
impl PySchemaBuilder {
    #[new]
    fn new() -> Self {
        PySchemaBuilder {
            builder: SchemaBuilder::new(),
        }
    }

    fn add_field(
        &mut self,
        name: String,
        field_type: PyFieldType,
        nullability: PyNullability,
    ) -> PyResult<PyObject> {
        // This is a placeholder - need proper error handling
        match self.builder.add_field(&name, field_type.0, nullability.0) {
            Ok(builder) => {
                self.builder = builder;
                Ok(PyObject::from(())) // Return self for chaining
            }
            Err(_) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Failed to add field")),
        }
    }

    fn build(&self) -> PyResult<PySchema> {
        match self.builder.build() {
            Ok(schema) => Ok(PySchema { schema }),
            Err(_) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Failed to build schema")),
        }
    }
}

/// Schema wrapper
#[pyclass]
struct PySchema {
    schema: Schema,
}

#[pymethods]
impl PySchema {
    #[getter]
    fn column_count(&self) -> usize {
        self.schema.fields.len()
    }
}

/// File writer for Python with high-level API
#[pyclass]
struct PyFileWriter {
    writer: Option<FileWriter<std::fs::File>>,
    schema: PySchema,
}

#[pymethods]
impl PyFileWriter {
    #[new]
    fn new(path: String, schema: PySchema) -> PyResult<Self> {
        match FileWriter::new(std::path::Path::new(&path), schema.schema.clone()) {
            Ok(writer) => Ok(PyFileWriter { writer: Some(writer), schema }),
            Err(_) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>("Failed to create writer")),
        }
    }

    /// Write a row using Python objects (high-level API)
    fn write_row_py(&mut self, row: Vec<PyObject>, py: Python) -> PyResult<()> {
        if row.len() != self.schema.schema.fields.len() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Row has {} columns but schema has {}", row.len(), self.schema.schema.fields.len())
            ));
        }

        let mut serialized_row = Vec::new();
        for (i, obj) in row.iter().enumerate() {
            let field_type = &self.schema.schema.fields[i].field_type;
            let bytes = serialize_py_object(obj, field_type, py)?;
            serialized_row.push(bytes);
        }

        self.write_row(serialized_row)
    }

    /// Write a row using raw bytes (low-level API)
    fn write_row(&mut self, row: Vec<Vec<u8>>) -> PyResult<()> {
        if let Some(ref mut writer) = self.writer {
            match writer.write_row(row) {
                Ok(_) => Ok(()),
                Err(_) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>("Failed to write row")),
            }
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Writer already finished"))
        }
    }

    fn finish(&mut self) -> PyResult<()> {
        if let Some(writer) = self.writer.take() {
            match writer.finish() {
                Ok(_) => Ok(()),
                Err(_) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>("Failed to finish writing")),
            }
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Writer already finished"))
        }
    }
}

/// Serialize Python object to bytes based on field type
fn serialize_py_object(obj: &PyObject, field_type: &FieldType, py: Python) -> PyResult<Vec<u8>> {
    match field_type {
        FieldType::Int64 => {
            let value: i64 = obj.extract(py)?;
            Ok(value.to_le_bytes().to_vec())
        }
        FieldType::Int32 => {
            let value: i32 = obj.extract(py)?;
            Ok(value.to_le_bytes().to_vec())
        }
        FieldType::Float64 => {
            let value: f64 = obj.extract(py)?;
            Ok(value.to_le_bytes().to_vec())
        }
        FieldType::Float32 => {
            let value: f32 = obj.extract(py)?;
            Ok(value.to_le_bytes().to_vec())
        }
        FieldType::Boolean => {
            let value: bool = obj.extract(py)?;
            Ok(vec![if value { 1 } else { 0 }])
        }
        FieldType::String => {
            let value: String = obj.extract(py)?;
            let mut result = Vec::new();
            let bytes = value.as_bytes();
            result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            result.extend_from_slice(bytes);
            Ok(result)
        }
        _ => Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            format!("Unsupported field type: {:?}", field_type)
        )),
    }
}

/// File reader for Python
#[pyclass]
struct PyFileReader {
    reader: Option<FileReader<std::fs::File>>,
}

/// Field type enum for Python
#[pyclass]
struct PyFieldType(FieldType);

#[pymethods]
impl PyFieldType {
    #[classattr]
    const INT64: Self = Self(FieldType::Int64);

    #[classattr]
    const STRING: Self = Self(FieldType::String);

    #[classattr]
    const FLOAT64: Self = Self(FieldType::Float64);
}

/// Nullability enum for Python
#[pyclass]
struct PyNullability(Nullability);

#[pymethods]
impl PyNullability {
    #[classattr]
    const REQUIRED: Self = Self(Nullability::Required);

    #[classattr]
    const OPTIONAL: Self = Self(Nullability::Optional);
}