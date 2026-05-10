# Deprecated API Reference

This file has been superseded by the new documentation structure. Please use `docs/SDKS.md` and `docs/FORMAT_SPEC.md` for current API and format guidance.

---

## Core Modules

### 1. Schema Module

#### `SchemaBuilder`

```rust
pub struct SchemaBuilder { ... }
```

**Methods:**

##### `new() -> SchemaBuilder`
Create a new schema builder.

```rust
let builder = SchemaBuilder::new();
```

##### `add_field(name: &str, field_type: FieldType, nullability: Nullability) -> Result<Self>`
Add a field to the schema.

**Parameters:**
- `name`: Field name (must be unique, non-empty)
- `field_type`: Data type from `FieldType` enum
- `nullability`: Nullability constraint

**Returns:** `Result<Self>` - Self for method chaining

```rust
let builder = SchemaBuilder::new()
    .add_field("user_id", FieldType::Int64, Nullability::Required)?
    .add_field("email", FieldType::String, Nullability::Optional)?;
```

**Errors:**
- `SchemaError::DuplicateField` - Field name already exists
- `SchemaError::InvalidFieldName` - Empty or invalid name
- `SchemaError::TooManyFields` - Exceeds maximum field limit

##### `build() -> Result<Schema>`
Build the schema.

```rust
let schema = builder.build()?;
```

**Returns:** `Result<Schema>` - Finalized, immutable schema

### 2. Writer Module

#### `FileWriter`

```rust
pub struct FileWriter { ... }
```

**Methods:**

##### `new(path: &Path, schema: Schema) -> Result<Self>`
Create a new file writer.

**Parameters:**
- `path`: Output file path
- `schema`: Schema for columns

**Returns:** `Result<Self>` - New writer instance

```rust
let writer = FileWriter::new(
    Path::new("output.qrd"),
    schema
)?;
```

**Errors:**
- `IoError` - Cannot create file
- `PermissionError` - No write permission

##### `write_row(values: Vec<Vec<u8>>) -> Result<()>`
Write a single row.

**Parameters:**
- `values`: Column values in order

**Returns:** `Result<()>`

```rust
writer.write_row(vec![
    serialize_int(42),
    serialize_string("test"),
])?;
```

**Errors:**
- `ValueError` - Wrong number of columns
- `IoError` - Write failed

##### `write_rows(rows: &[Vec<Vec<u8>>]) -> Result<()>`
Write multiple rows in batch.

**Parameters:**
- `rows`: Rows to write

**Returns:** `Result<()>`

```rust
let rows = vec![
    vec![serialize_int(1), serialize_string("a")],
    vec![serialize_int(2), serialize_string("b")],
];
writer.write_rows(&rows)?;
```

##### `finish() -> Result<()>`
Finalize and close the file.

Must be called to complete the write operation.

```rust
writer.finish()?;
```

**Important:** File is not valid until `finish()` is called.

### 3. Reader Module

#### `FileReader`

```rust
pub struct FileReader { ... }
```

**Methods:**

##### `new(path: &Path) -> Result<Self>`
Open a QRD file for reading.

**Parameters:**
- `path`: File path

**Returns:** `Result<Self>` - Reader instance

```rust
let reader = FileReader::new(Path::new("input.qrd"))?;
```

**Errors:**
- `IoError` - Cannot open file
- `FormatError` - Invalid QRD file

##### `schema() -> &Schema`
Get the file schema.

```rust
let schema = reader.schema();
println!("Fields: {}", schema.fields.len());
```

##### `row_count() -> u64`
Get total number of rows.

```rust
let count = reader.row_count();
println!("Total rows: {}", count);
```

##### `read_all() -> Result<Vec<Vec<Vec<u8>>>>`
Read all data.

**Returns:** `Result<Vec<...>>` - All rows with all columns

```rust
let all_data = reader.read_all()?;
for row in all_data {
    for col in row {
        // Process column data
    }
}
```

**Performance:** Loads entire file into memory.

##### `read_decoded_row_group(index: usize) -> Result<Vec<Vec<u8>>>`
Read and decode a specific row group.

**Parameters:**
- `index`: Row group index (0-based)

**Returns:** `Result<Vec<...>>` - Decoded columns

```rust
let first_group = reader.read_decoded_row_group(0)?;
```

**Performance:** Decodes entire row group into memory.

##### `read_encoded_row_group(index: usize) -> Result<Vec<Vec<u8>>>`
Read a row group without decoding.

**Returns:** `Result<Vec<...>>` - Encoded column bytes

```rust
let encoded = reader.read_encoded_row_group(0)?;
```

**Performance:** Faster if you cache encoded data.

### 4. Encryption Module

#### `EncryptionConfig`

```rust
pub struct EncryptionConfig {
    pub key: Vec<u8>,
    pub salt: Option<Vec<u8>>,
}
```

**Methods:**

##### `new(key: Vec<u8>) -> Result<Self>`
Create config with raw key.

**Parameters:**
- `key`: 32-byte (256-bit) key

**Returns:** `Result<Self>`

```rust
let key = vec![0u8; 32];
let config = EncryptionConfig::new(key)?;
```

**Errors:**
- `ConfigError` - Key is not 32 bytes

##### `with_salt(key: Vec<u8>, salt: Vec<u8>) -> Result<Self>`
Create config with key and salt.

**Parameters:**
- `key`: 32-byte key
- `salt`: 32-byte salt

**Returns:** `Result<Self>`

```rust
let config = EncryptionConfig::with_salt(key, salt)?;
```

##### `generate_key() -> Vec<u8>`
Generate a random 256-bit key.

```rust
let key = EncryptionConfig::generate_key();
```

##### `generate_salt() -> Vec<u8>`
Generate a random 32-byte salt.

```rust
let salt = EncryptionConfig::generate_salt();
```

##### `derive_from_password(password: &str, salt: &[u8]) -> Result<Self>`
Derive key from password using HKDF.

**Parameters:**
- `password`: User password
- `salt`: 32-byte salt (not secret, should be random)

**Returns:** `Result<Self>`

```rust
let password = "secure_password_123";
let salt = EncryptionConfig::generate_salt();
let config = EncryptionConfig::derive_from_password(password, &salt)?;
```

**Performance:** ~300-500ms (intentionally slow for security)

#### Functions

##### `encrypt(data: &[u8], config: &EncryptionConfig) -> Result<Vec<u8>>`
Encrypt data with AES-256-GCM.

**Parameters:**
- `data`: Plaintext bytes
- `config`: Encryption configuration

**Returns:** `Result<Vec<u8>>` - Ciphertext with nonce

**Algorithm:** AES-256-GCM with random nonce

```rust
let ciphertext = encrypt(plaintext, &config)?;
```

**Output Format:**
```
[Nonce (12B)] [Ciphertext] [Tag (16B)]
```

##### `decrypt(data: &[u8], config: &EncryptionConfig) -> Result<Vec<u8>>`
Decrypt data.

**Parameters:**
- `data`: Ciphertext from `encrypt()`
- `config`: Same configuration as encryption

**Returns:** `Result<Vec<u8>>` - Plaintext

```rust
let plaintext = decrypt(&ciphertext, &config)?;
```

**Errors:**
- `EncryptionError` - Wrong key or corrupted ciphertext
- `AuthenticationError` - Tag verification failed

### 5. Error Correction Module

#### `EccConfig`

```rust
pub struct EccConfig {
    pub parity_chunks: usize,
    pub chunk_size: usize,
}
```

**Methods:**

##### `with_chunk_size(parity: usize, chunk_size: usize) -> Result<Self>`
Create ECC configuration.

**Parameters:**
- `parity`: Number of parity chunks (recoverable losses)
- `chunk_size`: Bytes per chunk (typically 256-4096)

**Returns:** `Result<Self>`

```rust
// Can recover from 2 chunk losses out of n+2 total
let config = EccConfig::with_chunk_size(2, 1024)?;
```

**Error Correction Capacity:**
- 1 parity → recover 1 chunk loss
- 2 parity → recover 2 losses
- 3 parity → recover 3 losses
- n parity → recover up to n losses

Examples:
```rust
EccConfig::with_chunk_size(1, 256)?   // Recover 1/n (12.5% overhead)
EccConfig::with_chunk_size(2, 512)?   // Recover 2 (25% overhead)
EccConfig::with_chunk_size(3, 1024)?  // Recover 3 (50% overhead)
```

#### `EccCodec`

```rust
pub struct EccCodec { ... }
```

**Methods:**

##### `new(config: EccConfig) -> Result<Self>`
Create encoder/decoder.

```rust
let codec = EccCodec::new(config)?;
```

##### `encode(data: &[u8]) -> Result<EncodedShards>`
Encode data with ECC.

**Parameters:**
- `data`: Original data

**Returns:** `Result<EncodedShards>` - Data + parity chunks

```rust
let encoded = codec.encode(&original_data)?;
```

**Output:** `n` data shards + `parity` parity shards

#### Functions

##### `decode_and_recover(shards: &[Option<Vec<u8>>], config: &EccConfig) -> Result<Vec<u8>>`
Recover original data from shards.

**Parameters:**
- `shards`: Shards with Some/None options
- `config`: Same ECC config as encoding

**Returns:** `Result<Vec<u8>>` - Original data

```rust
let mut shards = encoded.shards_as_options();
shards[0] = None; // Simulate loss

let recovered = decode_and_recover(&shards, &config)?;
assert_eq!(recovered, original_data);
```

**Requirements:**
- At least `n` shards must be present (where `n` = data chunk count)
- If fewer shards present, recovery fails

### 6. Validation Module

#### `CorruptionDetector`

```rust
pub struct CorruptionDetector { ... }
```

**Methods:**

##### `new() -> Self`
Create a corruption detector.

```rust
let detector = CorruptionDetector::new();
```

##### `detect_magic_corruption(data: &[u8]) -> Vec<CorruptionReport>`
Check file magic number and version.

```rust
let file_data = std::fs::read("data.qrd")?;
let reports = detector.detect_magic_corruption(&file_data);
if !reports.is_empty() {
    println!("File corruption detected!");
}
```

#### Functions

##### `calculate_crc32(data: &[u8]) -> u32`
Calculate CRC32 checksum.

```rust
let checksum = calculate_crc32(&data);
```

---

## Data Type Details

### Numeric Types

| Type | Size | Range | Example |
|------|------|-------|---------|
| Int8 | 1 byte | -128 to 127 | -50 |
| Int16 | 2 bytes | -32,768 to 32,767 | -1000 |
| Int32 | 4 bytes | -2³¹ to 2³¹-1 | -100_000 |
| Int64 | 8 bytes | -2⁶³ to 2⁶³-1 | -10_000_000_000 |
| UInt8 | 1 byte | 0 to 255 | 200 |
| UInt16 | 2 bytes | 0 to 65,535 | 60_000 |
| UInt32 | 4 bytes | 0 to 2³²-1 | 4_000_000_000 |
| UInt64 | 8 bytes | 0 to 2⁶⁴-1 | 18_000_000_000_000 |
| Float32 | 4 bytes | ±3.4×10³⁸ | 3.14159 |
| Float64 | 8 bytes | ±1.8×10³⁰⁸ | 3.141592653589793 |

### String/Text Types

| Type | Storage | Example |
|------|---------|---------|
| String | UTF-8 + length prefix | "Hello, World!" |
| Blob | Binary + length prefix | bytes: [72, 101, 108, 108, 111] |

### Temporal Types

| Type | Size | Range | Unit |
|------|------|-------|------|
| Timestamp | 8 bytes | 1970-2262 | Microseconds |
| Date | 4 bytes | 1970-2200 | Days |
| Time | 8 bytes | 00:00:00-23:59:59 | Microseconds |
| Duration | 8 bytes | -2⁶³ to 2⁶³-1 | Microseconds |

### Special Types

| Type | Description | Example |
|------|-------------|---------|
| Boolean | 1 byte, 0=false, 1=true | true |
| Enum | String with predefined values | "PENDING" |
| UUID | 16 bytes fixed | "550e8400-e29b-41d4-a716-446655440000" |
| Decimal | Fixed-point number | 123.45 |

---

## Encoding Algorithms

### PLAIN (Default)
- **Best for:** Unpredictable data
- **Overhead:** 0%
- **Speed:** Baseline (100%)

### RLE (Run-Length Encoding)
- **Best for:** Repetitive values
- **Overhead:** -50% to +10% (depends on data)
- **Speed:** 90%

Encoding: `[run_length][value] [run_length][value]`

### DELTA_BINARY
- **Best for:** Sorted/monotonic integers
- **Overhead:** -60% to +5%
- **Speed:** 85%

Encodes differences between consecutive values.

### BIT_PACKED
- **Best for:** Small integers, booleans
- **Overhead:** -75% to +5%
- **Speed:** 80%

Packs multiple values into fewer bits.

### DICTIONARY_RLE
- **Best for:** Low-cardinality strings
- **Overhead:** -90% to +10% (depends on cardinality)
- **Speed:** 75%

Uses dictionary + RLE for repeated values.

### BYTE_STREAM_SPLIT
- **Best for:** Floating-point data
- **Overhead:** -40% to +10%
- **Speed:** 85%

Splits bytes by position for better compression.

### DELTA_BYTE_ARRAY
- **Best for:** Sorted strings
- **Overhead:** -70% (on sorted data)
- **Speed:** 80%

Encodes differences between consecutive strings.

---

## Error Types

### SchemaError
```rust
pub enum SchemaError {
    DuplicateField,
    InvalidFieldName,
    TooManyFields,
    UnsupportedType,
}
```

### IoError
```rust
pub enum IoError {
    FileNotFound,
    PermissionDenied,
    InvalidPath,
}
```

### ValueError
```rust
pub enum ValueError {
    WrongColumnCount,
    InvalidType,
    TruncatedData,
}
```

### EncryptionError
```rust
pub enum EncryptionError {
    InvalidKeySize,
    InvalidSaltSize,
    EncryptionFailed,
    AuthenticationFailed,
}
```

### EccError
```rust
pub enum EccError {
    InvalidConfig,
    InsufficientShards,
    RecoveryFailed,
    CorruptedShards,
}
```

---

## Performance Characteristics

### Memory Usage
- Schema: O(fields)
- Open reader: O(1)
- Read row group: O(row_group_size)
- Encryption: O(data_size)
- ECC: O(data_size + parity_size)

### Time Complexity
| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Open file | O(1) | Deferred to first read |
| Read header | O(1) | Fixed size |
| Read footer | O(1) | Via seek |
| Decrypt | O(n) | Single pass |
| ECC encode | O(n * parity) | Linear |
| ECC decode | O(n log n) | Depends on implementation |

---

## Version Compatibility

### Format Versions
- **1.0:** Initial release (stable)
- **1.1:** Partial reads support (stable)
- **1.2:** Encryption & ECC (current)

### Forward Compatibility
- Unknown encodings: Skip gracefully
- Unknown compression: Error with helpful message
- Unknown types: Treat as opaque binary

### Backward Compatibility
- v1.2 readers can open v1.0 files ✓
- v1.0 readers cannot open v1.2 encrypted files (expected)

---

**Version:** 1.2.0  
**Last Updated:** May 2026
