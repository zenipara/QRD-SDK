# QRD Go Bindings

Go bindings for the QRD columnar binary format using CGO.

## Installation

```bash
go get github.com/zenipara/QRD-SDK/sdk/go
```

## Building

### Requirements

1. Rust toolchain (to build the FFI library)
2. C compiler (gcc/clang) for CGO
3. Standard Go toolchain

### Build Process

First, build the Rust FFI library:

```bash
cd ../..  # Go to repository root
cargo build --release -p qrd-ffi
```

Then verify `libqrd_ffi.so` (Linux) or `libqrd_ffi.dylib` (macOS) exists:

```bash
ls target/release/libqrd_ffi.*
```

Then build the Go bindings:

```bash
cd sdk/go
go build ./...
go test ./...
```

## Usage

### Basic Example

```go
package main

import (
    "fmt"
    "log"

    qrd "github.com/zenipara/QRD-SDK/sdk/go"
)

func main() {
    // Create schema
    schema, err := qrd.NewSchema().
        AddField("id", qrd.FieldTypeInt64, qrd.NullabilityRequired, "").
        AddField("name", qrd.FieldTypeString, qrd.NullabilityOptional, "").
        AddField("score", qrd.FieldTypeFloat64, qrd.NullabilityOptional, "").
        Build()
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Schema ID: %d\n", schema.ID())

    // Create writer
    writer, err := qrd.NewFileWriter(schema)
    if err != nil {
        log.Fatal(err)
    }

    // Write some data
    err = writer.WriteRow([]interface{}{int64(1), "Alice", 95.5})
    if err != nil {
        log.Fatal(err)
    }

    err = writer.WriteRow([]interface{}{int64(2), "Bob", 87.2})
    if err != nil {
        log.Fatal(err)
    }

    // Finish writing
    data, err := writer.Finish()
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Written %d bytes\n", len(data))

    // Create reader
    reader, err := qrd.NewFileReader(data)
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Row count: %d\n", reader.RowCount())

    // Read all rows
    rows, err := reader.ReadAllRows()
    if err != nil {
        log.Fatal(err)
    }

    for i, row := range rows {
        fmt.Printf("Row %d: %v\n", i, row)
    }
}
```

### Schema Definition

```go
schema, err := qrd.NewSchema().
    AddField("user_id", qrd.FieldTypeInt64, qrd.NullabilityRequired, "User identifier").
    AddField("email", qrd.FieldTypeString, qrd.NullabilityRequired, "").
    AddField("age", qrd.FieldTypeInt32, qrd.NullabilityOptional, "").
    AddField("balance", qrd.FieldTypeDecimal, qrd.NullabilityOptional, "").
    AddField("active", qrd.FieldTypeBoolean, qrd.NullabilityRequired, "").
    Build()
```

### Supported Types

- `FieldTypeBoolean` - Boolean values
- `FieldTypeInt8/16/32/64` - Signed integers
- `FieldTypeUint8/16/32/64` - Unsigned integers
- `FieldTypeFloat32/64` - Floating point numbers
- `FieldTypeString` - UTF-8 strings
- `FieldTypeBlob` - Binary data
- `FieldTypeTimestamp/Date/Time/Duration` - Temporal types
- `FieldTypeUUID` - UUID values
- `FieldTypeDecimal` - Decimal numbers
- `FieldTypeEnum` - Enumerated values

### Nullability

- `NullabilityRequired` - Field must be present
- `NullabilityOptional` - Field may be null
- `NullabilityRepeated` - Field is an array

## Testing

Run the tests:

```bash
go test
```

## Requirements

- Go 1.21 or later
- Rust toolchain (for building the FFI library)
- C compiler (GCC/Clang)

## Architecture

The Go bindings use CGO to interface with the Rust QRD core library through a C FFI layer. This provides:

- Memory safety through Rust's ownership system
- High performance columnar processing
- Type safety at the Go level
- Automatic resource management with finalizers