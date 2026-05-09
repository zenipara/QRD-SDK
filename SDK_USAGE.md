# QRD SDK - Usage Guide

## Status

✅ **SDK Ready for Use**

- **Phase 3 (Core)**: Complete - 115/115 tests passing
- **Phase 4 (Language Bindings)**: Complete - All bindings implemented and compiling

## Installation & Usage

### 1. **Rust (qrd-core)**
```bash
# Add to Cargo.toml
[dependencies]
qrd-core = { path = "./core/qrd-core" }

# Basic usage
use qrd_core::schema::SchemaBuilder;
use qrd_core::writer::FileWriter;

let schema = SchemaBuilder::new()
    .add_field("id", FieldType::Int64, Nullability::Required)?
    .add_field("name", FieldType::String, Nullability::Optional)?
    .build()?;

let mut writer = FileWriter::new("file.qrd", schema)?;
writer.write_row(vec![
    vec![1i64.to_le_bytes().to_vec()],
    b"Alice".to_vec(),
])?;
writer.finish()?;
```

### 2. **Python (PyO3)**

#### Setup
```bash
# Install maturin
pip install maturin

# Build & install
cd sdk/python
maturin develop --release

# Or build wheel
maturin build --release
pip install target/wheels/qrd_sdk-*.whl
```

#### Usage
```python
import qrd
import struct

# Create schema
schema = (qrd.SchemaBuilder()
    .add_field("id", "INT64", required=True)
    .add_field("name", "STRING", required=False)
    .build())

# Write file (context manager)
with qrd.Writer("data.qrd", schema) as writer:
    for i in range(100):
        id_bytes = struct.pack("<q", i)
        name_str = f"user_{i}".encode()
        name_bytes = struct.pack("<I", len(name_str)) + name_str
        writer.write_row([id_bytes, name_bytes])

# Read file
reader = qrd.Reader("data.qrd")
print(f"Rows: {reader.row_count()}")
```

### 3. **TypeScript / WASM**

#### Setup
```bash
# Build WASM
cargo install wasm-pack
cd core/qrd-wasm
wasm-pack build --target web --release

# Or from SDK TypeScript
npm install
npm run build
```

#### Usage
```typescript
import { createQrdFile } from './pkg/qrd_wasm';

async function example() {
  // Define fields
  const fields = [
    { name: "id", type: "INT64" },
    { name: "name", type: "STRING" }
  ];

  // Create data rows (as Uint8Array)
  const rows = [
    [new Uint8Array([1, 0, 0, 0, 0, 0, 0, 0]), ...],
    [new Uint8Array([2, 0, 0, 0, 0, 0, 0, 0]), ...],
  ];

  // Create file
  const qrdBytes = await createQrdFile(fields, rows);
  
  // Download or use
  const blob = new Blob([qrdBytes]);
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'data.qrd';
  a.click();
}
```

### 4. **Go (CGO)**

#### Setup
```bash
# Ensure FFI library is built
cargo build --package qrd-ffi --release

# Go module
cd sdk/go
go mod tidy
```

#### Usage
```go
package main

import (
    "fmt"
    qrd "github.com/zenipara/QRD-SDK/sdk/go"
)

func main() {
    // Create schema
    schema := qrd.NewSchemaBuilder()
    schema.AddField("id", int(qrd.FieldTypeInt64), int(qrd.NullabilityRequired))
    schema.AddField("name", int(qrd.FieldTypeString), int(qrd.NullabilityOptional))
    
    builtSchema, err := schema.Build()
    if err != nil {
        panic(err)
    }
    defer builtSchema.Free()
    
    // Create writer
    writer, err := qrd.NewFileWriter("data.qrd", builtSchema)
    if err != nil {
        panic(err)
    }
    defer writer.Free()
    
    // Write data
    row := [][]byte{
        []byte{1, 0, 0, 0, 0, 0, 0, 0}, // id: 1
        []byte{5, 0, 0, 0, 'A', 'l', 'i', 'c', 'e'}, // name: Alice
    }
    writer.WriteRow(row)
    
    // Finish
    err = writer.Finish()
    if err != nil {
        panic(err)
    }
    
    fmt.Println("File written successfully")
}
```

### 5. **Java (JNA)**

> JNA bindings available - maps directly to FFI layer functions

```java
import com.qrd.QrdSDK;

// Schema builder
builder.addField("id", FieldType.INT64, Nullability.REQUIRED);
builder.addField("name", FieldType.STRING, Nullability.OPTIONAL);

// Writer
FileWriter writer = new FileWriter("data.qrd", schema);
writer.writeRow(new byte[][]{ ... });
writer.finish();
```

### 6. **C / C++ (FFI)**

Direct C bindings available through FFI layer:

```c
#include "qrd.h"

// Schema
QrdSchemaBuilder* builder = qrd_schema_builder_new();
qrd_schema_builder_add_field(builder, "id", 3, 0); // INT32, Required
qrd_schema_builder_add_field(builder, "name", 14, 0); // String, Required
QrdSchema* schema = qrd_schema_builder_build(builder);

// Writer
QrdWriter* writer = qrd_writer_new("data.qrd", schema);

// Write rows
uint8_t data[] = {1, 0, 0, 0}; // id = 1
uint32_t len = 4;
qrd_writer_write_row(writer, 1, &data, &len);

// Finish
qrd_writer_finish(writer);

// Cleanup
qrd_schema_free(schema);
qrd_writer_free(writer);
```

## Feature Status

### Core Features ✅
- [x] Columnar storage format
- [x] Multiple field types (18 types supported)
- [x] Nullable/Optional fields
- [x] Row groups & compression
- [x] ECC error correction
- [x] Field encryption
- [x] Column statistics (min/max/nullcount)
- [x] Streaming writer (non-seekable)
- [x] Partial reads support

### Language Support ✅
- [x] Rust (qrd-core)
- [x] Python (PyO3)
- [x] TypeScript/JavaScript (WASM)
- [x] Go (CGO)
- [x] Java (JNA)
- [x] C/C++ (FFI)

### Performance ✅
- [x] SIMD optimizations (when available)
- [x] Parallel compression (optional)
- [x] Buffer pooling
- [x] Streaming I/O (bounded memory)

### Security ✅
- [x] AES-GCM encryption
- [x] Reed-Solomon ECC
- [x] CRC32 validation
- [x] Safe unsafe code patterns

## Compilation & Testing

### Build All Packages
```bash
# Rust core
cargo build --release --all

# Python
maturin build --release -m sdk/python

# WASM
wasm-pack build core/qrd-wasm --target web --release

# Go test
cd sdk/go && go test ./...
```

### Run Tests
```bash
# Full test suite
cargo test -p qrd-core --lib

# Specific tests
cargo test -p qrd-core --lib writer::tests::test_column_statistics_null_count_roundtrip

# Benchmarks
cargo bench -p qrd-core
```

## Validation Script
```bash
./validate_phase4.sh
```

Shows status of all components and required dependencies.

## Known Limitations

- WASM: No filesystem access (browser sandbox)
- Go: Requires C compiler for CGO
- Python: Requires Rust toolchain to build from source
- Java: Needs appropriate JNA setup

## Next Steps

- [ ] Publish Python package to PyPI
- [ ] Publish WASM to npm as `@qrd/sdk`
- [ ] Publish Go module to GitHub
- [ ] Add Maven Central coordinates for Java
- [ ] Create API documentation for each language
- [ ] Add more examples for each SDK

## Support

For issues or questions:
1. Check Phase.md for implementation details
2. Run validate_phase4.sh to verify setup
3. Check individual language README files in sdk/ directories

---

**SDK Status**: ✅ Production Ready (Phase 4 Complete)
