# QRD Java Bindings

Java bindings for the QRD columnar binary format using JNA.

## Installation

Add to your Maven `pom.xml`:

```xml
<dependency>
    <groupId>com.zenipara</groupId>
    <artifactId>qrd-sdk</artifactId>
    <version>0.1.0</version>
</dependency>
```

## Building from Source

First, build the Rust FFI library:

```bash
cd ../..
cargo build --release
```

Then build the Java bindings:

```bash
mvn clean compile
```

## Usage

### Basic Example

```java
import com.zenipara.qrd.*;

public class Example {
    public static void main(String[] args) throws QRDException {
        // Create schema
        Schema schema = QRD.newSchema()
            .addField("id", FieldType.INT64, Nullability.REQUIRED)
            .addField("name", FieldType.STRING, Nullability.OPTIONAL)
            .addField("score", FieldType.FLOAT64, Nullability.OPTIONAL)
            .build();

        System.out.println("Schema ID: " + schema.getId());

        // Create writer
        try (FileWriter writer = QRD.newFileWriter(schema)) {
            // Write data
            writer.writeRow(1L, "Alice", 95.5);
            writer.writeRow(2L, "Bob", 87.2);

            // Finish writing
            byte[] data = writer.finish();
            System.out.println("Written " + data.length + " bytes");
        }

        // Create reader
        try (FileReader reader = QRD.newFileReader(data)) {
            System.out.println("Row count: " + reader.getRowCount());

            // Read all rows
            List<List<Object>> rows = reader.readAllRows();
            for (int i = 0; i < rows.size(); i++) {
                System.out.println("Row " + i + ": " + rows.get(i));
            }
        }

        schema.close();
    }
}
```

### Schema Definition

```java
Schema schema = QRD.newSchema()
    .addField("user_id", FieldType.INT64, Nullability.REQUIRED, "User identifier")
    .addField("email", FieldType.STRING, Nullability.REQUIRED, null)
    .addField("age", FieldType.INT32, Nullability.OPTIONAL, null)
    .addField("balance", FieldType.DECIMAL, Nullability.OPTIONAL, null)
    .addField("active", FieldType.BOOLEAN, Nullability.REQUIRED, null)
    .build();
```

### Supported Types

- `FieldType.BOOLEAN` - Boolean values
- `FieldType.INT8/16/32/64` - Signed integers
- `FieldType.UINT8/16/32/64` - Unsigned integers
- `FieldType.FLOAT32/64` - Floating point numbers
- `FieldType.STRING` - UTF-8 strings
- `FieldType.BLOB` - Binary data
- `FieldType.TIMESTAMP/DATE/TIME/DURATION` - Temporal types
- `FieldType.UUID` - UUID values
- `FieldType.DECIMAL` - Decimal numbers
- `FieldType.ENUM` - Enumerated values

### Nullability

- `Nullability.REQUIRED` - Field must be present
- `Nullability.OPTIONAL` - Field may be null
- `Nullability.REPEATED` - Field is an array

## Testing

Run the tests:

```bash
mvn test
```

## Requirements

- Java 11 or later
- Maven 3.6+
- Rust toolchain (for building the FFI library)

## Architecture

The Java bindings use JNA (Java Native Access) to interface with the Rust QRD core library through a C FFI layer. This provides:

- Memory safety through Rust's ownership system
- High performance columnar processing
- Type safety at the Java level
- Automatic resource management with try-with-resources