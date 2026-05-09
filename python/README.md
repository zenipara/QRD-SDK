# QRD Python Bindings

Python bindings for the QRD columnar binary format.

## Installation

```bash
pip install .
```

## Usage

### High-Level API (Recommended)

```python
import qrd

# Create schema
schema_builder = qrd.PySchemaBuilder()
schema_builder.add_field("id", qrd.PyFieldType.INT64, qrd.PyNullability.REQUIRED)
schema_builder.add_field("name", qrd.PyFieldType.STRING, qrd.PyNullability.OPTIONAL)
schema_builder.add_field("score", qrd.PyFieldType.FLOAT64, qrd.PyNullability.OPTIONAL)
schema_builder.add_field("active", qrd.PyFieldType.BOOLEAN, qrd.PyNullability.REQUIRED)
schema = schema_builder.build()

# Write data using Python objects
writer = qrd.PyFileWriter("data.qrd", schema)
writer.write_row_py([42, "Alice", 95.5, True])
writer.write_row_py([43, "Bob", 87.2, False])
writer.write_row_py([44, "Charlie", 91.8, True])
writer.finish()

print("Successfully wrote data to data.qrd")
```

### Low-Level API (Advanced)

```python
import qrd

# Create schema
schema_builder = qrd.PySchemaBuilder()
schema_builder.add_field("id", qrd.PyFieldType.INT64, qrd.PyNullability.REQUIRED)
schema_builder.add_field("name", qrd.PyFieldType.STRING, qrd.PyNullability.OPTIONAL)
schema = schema_builder.build()

# Write data using raw bytes
writer = qrd.PyFileWriter("data.qrd", schema)

# For INT64: convert to little-endian bytes
id_bytes = (42).to_bytes(8, byteorder='little', signed=True)
# For STRING: length prefix + UTF-8 bytes
name_str = "hello"
name_bytes = len(name_str).to_bytes(4, byteorder='little') + name_str.encode('utf-8')

writer.write_row([id_bytes, name_bytes])
writer.finish()
```

## Status

- ✅ Schema building API
- ✅ File writer with high-level Python object API
- ✅ File writer with low-level byte API
- ✅ Basic field types (INT64, INT32, FLOAT64, FLOAT32, BOOLEAN, STRING)
- ✅ Automatic serialization from Python objects
- ❌ File reader
- ❌ Full data type support (BLOB, DECIMAL, etc.)
- ❌ Null value handling