# QRD TypeScript Bindings

TypeScript bindings for the QRD columnar binary format using WebAssembly.

## Installation

```bash
npm install qrd-sdk
```

## Building from Source

To build the bindings from source:

```bash
# Install dependencies
npm install

# Build WebAssembly module
npm run build:wasm

# Build TypeScript bindings
npm run build
```

## Testing

Run the test suite:

```bash
npm test
```

## Usage

### Basic Example

```typescript
import { SchemaBuilder, FileWriter, FileReader, FieldType, Nullability } from 'qrd-sdk';

// Create schema
const schemaBuilder = new SchemaBuilder()
  .addField('id', FieldType.INT64, Nullability.REQUIRED)
  .addField('name', FieldType.STRING, Nullability.OPTIONAL)
  .addField('score', FieldType.FLOAT64, Nullability.OPTIONAL);

const schema = schemaBuilder.build();

// Write data
const writer = new FileWriter(schema);
writer.writeRow([42, 'Alice', 95.5]);
writer.writeRow([43, 'Bob', 87.2]);
const buffer = writer.finish();

// Read data
const reader = new FileReader(buffer);
console.log('Schema:', reader.getSchema());
console.log('Row count:', reader.getRowCount());
console.log('All rows:', reader.readAllRows());
```

### Browser Usage

The bindings work in both Node.js and browser environments:

```javascript
// In browser, load the WASM module first
import init, { SchemaBuilder, FileWriter, FileReader } from './pkg/qrd_wasm.js';

async function main() {
  // Initialize WASM
  await init();

  // Use the API as normal
  const schemaBuilder = new SchemaBuilder();
  // ... rest of the code
}
```

### Manual Serialization

```typescript
import { Serializer } from 'qrd-sdk';

// Serialize different types
const intBytes = Serializer.serializeInt64(42);
const floatBytes = Serializer.serializeFloat64(3.14);
const stringBytes = Serializer.serializeString('hello');

// Deserialize
const intValue = Serializer.deserializeInt64(intBytes);
const floatValue = Serializer.deserializeFloat64(floatBytes);
const stringValue = Serializer.deserializeString(stringBytes);
```

## Status

- ✅ Schema building API
- ✅ Basic file I/O (JSON-based for now)
- ✅ Data serialization utilities
- ✅ TypeScript type definitions
- ❌ WASM integration
- ❌ Full QRD binary format support
- ❌ Compression and encoding
- ❌ Streaming I/O

## Development

This is currently a TypeScript-only implementation for API design and testing. Future versions will integrate with the Rust WASM bindings for full performance.