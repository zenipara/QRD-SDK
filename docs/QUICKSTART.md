# Deprecated Quick Start Guide

This file has been superseded by the new documentation structure. Please use `README.md` and `docs/sdk/SDKS.md` for installation and quick start information.

## Installation

### Rust

```bash
# Add to Cargo.toml
[dependencies]
qrd-core = "0.1"
```

### Python

```bash
pip install qrd-sdk
```

### Node.js

```bash
npm install qrd-sdk
```

### Go

```bash
go get github.com/zenipara/QRD-SDK/sdk/go
```

## Basic Usage

### Rust

```rust
use qrd_core::prelude::*;
use std::fs::File;

fn main() -> Result<()> {
    // Define schema
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)?
        .add_field("name", FieldType::String, Nullability::Optional)?
        .add_field("score", FieldType::Float64, Nullability::Optional)?
        .build()?;

    println!("Schema ID: {}", schema.schema_id);

    // Write data
    let mut writer = FileWriter::new("output.qrd", schema)?;
    
    for i in 0..1000 {
        writer.write_row(vec![
            FieldType::Int64,
            FieldType::String,
            FieldType::Float64,
        ])?;
    }
    
    writer.finish()?;
    println!("Wrote 1000 rows");

    // Read data back
    let reader = FileReader::new("output.qrd")?;
    println!("Schema: {:?}", reader.schema());
    println!("Rows: {}", reader.row_count());

    Ok(())
}
```

### Python

```python
import qrd

# Define schema
schema = qrd.SchemaBuilder() \
    .add_field("id", qrd.FieldType.INT64, qrd.Nullability.REQUIRED) \
    .add_field("name", qrd.FieldType.STRING, qrd.Nullability.OPTIONAL) \
    .add_field("score", qrd.FieldType.FLOAT64, qrd.Nullability.OPTIONAL) \
    .build()

# Write data
writer = qrd.FileWriter("output.qrd", schema)

for i in range(1000):
    writer.write_row({
        "id": i,
        "name": f"User {i}",
        "score": 100.0 - (i % 100)
    })

writer.finish()

# Read data
reader = qrd.FileReader("output.qrd")
print(f"Schema: {reader.schema()}")
print(f"Rows: {reader.row_count()}")

for row in reader.rows():
    print(row)
    if reader.row_index() > 10:
        break
```

### TypeScript/JavaScript

```javascript
const qrd = require('qrd-sdk');

// Define schema
const schema = new qrd.SchemaBuilder()
    .addField("id", qrd.FieldType.INT64, qrd.Nullability.REQUIRED)
    .addField("name", qrd.FieldType.STRING, qrd.Nullability.OPTIONAL)
    .addField("score", qrd.FieldType.FLOAT64, qrd.Nullability.OPTIONAL)
    .build();

// Write data
const writer = new qrd.FileWriter("output.qrd", schema);

for (let i = 0; i < 1000; i++) {
    writer.writeRow({
        id: i,
        name: `User ${i}`,
        score: 100 - (i % 100)
    });
}

writer.finish();

// Read data
const reader = new qrd.FileReader("output.qrd");
console.log("Schema:", reader.schema());
console.log("Rows:", reader.rowCount());

for (const row of reader.rows()) {
    console.log(row);
}
```

### Go

```go
package main

import (
    "fmt"
    qrd "github.com/zenipara/QRD-SDK/sdk/go"
)

func main() {
    // Define schema
    schema := qrd.NewSchemaBuilder().
        AddField("id", qrd.FieldTypeInt64, qrd.NullabilityRequired).
        AddField("name", qrd.FieldTypeString, qrd.NullabilityOptional).
        AddField("score", qrd.FieldTypeFloat64, qrd.NullabilityOptional).
        Build()

    // Write data
    writer, _ := qrd.NewFileWriter("output.qrd", schema)
    defer writer.Close()

    for i := 0; i < 1000; i++ {
        writer.WriteRow(map[string]interface{}{
            "id":    i,
            "name":  fmt.Sprintf("User %d", i),
            "score": 100.0 - float64(i%100),
        })
    }

    writer.Finish()

    // Read data
    reader, _ := qrd.NewFileReader("output.qrd")
    defer reader.Close()

    fmt.Printf("Rows: %d\n", reader.RowCount())

    for row := range reader.Rows() {
        fmt.Println(row)
    }
}
```

## Common Tasks

### Write Large Dataset

```rust
use qrd_core::prelude::*;
use std::io::BufRead;

fn main() -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("timestamp", FieldType::Timestamp, Nullability::Required)?
        .add_field("event", FieldType::String, Nullability::Required)?
        .add_field("user_id", FieldType::Int64, Nullability::Optional)?
        .add_field("value", FieldType::Float64, Nullability::Optional)?
        .build()?;

    let mut writer = FileWriter::with_config(
        "events.qrd",
        schema,
        WriterConfig {
            row_group_size: 1_000_000,  // 1M rows per group
            compression_level: 3,
        }
    )?;

    // Read from CSV or other source
    for (i, line) in std::io::stdin().lock().lines().enumerate() {
        let parts: Vec<&str> = line?.split(',').collect();
        
        writer.write_row(vec![
            FieldType::Timestamp,
            FieldType::String,
            FieldType::Int64,
            FieldType::Float64,
        ])?;

        if (i + 1) % 100_000 == 0 {
            println!("Wrote {} rows", i + 1);
        }
    }

    writer.finish()?;
    Ok(())
}
```

### Read Specific Columns

```rust
let reader = FileReader::new("data.qrd")?;

// Read only "name" and "score" columns
let columns = reader.read_columns(&[1, 2])?;  // Column indices

for (name, score) in columns.iter().zip(columns[1].iter()) {
    println!("{}: {}", name, score);
}
```

### Verify File Integrity

```rust
let mut reader = FileReader::new("data.qrd")?;
match reader.verify() {
    Ok(()) => println!("File is valid"),
    Err(e) => eprintln!("Corrupted: {}", e),
}
```

## Performance Tips

### For Writing

```rust
// Use larger row groups for batch writes
let config = WriterConfig {
    row_group_size: 10_000_000,  // 10M rows
    compression_level: 10,         // High compression
};

// Buffer rows in memory, then write in chunks
for batch in data.chunks(10_000) {
    for row in batch {
        writer.write_row(row)?;
    }
}
```

### For Reading

```rust
// Use streaming for large files
for chunk in reader.rows()? {
    process_chunk(chunk)?;
}

// Use partial reads for selective columns
let columns = reader.read_columns(&[0, 2,  5])?;
```

## Troubleshooting

### "Invalid magic number"

```
File is not a QRD file. Check the file path and format.
```

### "Unsupported version"

```
File was created with a newer QRD version.
Upgrade your SDK.
```

### "CRC mismatch"

```
File is corrupted or incomplete.
Check disk integrity.
```

### "Schema mismatch"

```
Schema doesn't match file.
Recheck schema definition or use a different file.
```

---

**More examples in `examples/` directory.**

For detailed API documentation, see [API Reference](./API.md).
