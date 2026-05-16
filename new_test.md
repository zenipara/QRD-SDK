Berikut prompt siap pakai untuk GitHub Copilot Chat di Codespace:

---

```
You are a senior Rust engineer and QA architect. Your task is to generate exactly 600 additional test cases for the QRD-SDK codebase. These tests must cover every critical gap, architectural boundary, and potential regression point identified in the audit.

## CONTEXT

QRD-SDK is a columnar binary container format (like Parquet but edge/WASM-native) built in Rust. The current test suite has the following known failures and gaps:

1. CI is 73% failing (11/15 checks fail)
2. Active merge conflict in security_test.rs
3. 14 compile errors in encryption_edge_cases_test.rs (type mismatch [u8; 32] vs Vec<u8>)
4. 88 clippy errors in tests, 54 in lib
5. Dead SIMD code (memcpy_simd, xor_simd, delta_encode_i32_simd, delta_decode_i32_simd, count_bytes_simd never used)
6. FieldType::Bool referenced but does not exist in enum
7. assert!(true) placeholder tests in partial_reader.rs
8. Schema ID uses only 4 bytes of SHA-256 (collision risk)
9. No cross-SDK determinism tests verified running
10. No fuzz targets actually present despite being mentioned in docs
11. cargo-audit not installed (no CVE scanning)
12. Footer CRC32 only (no cryptographic protection for plaintext footer)

## ARCHITECTURE TO TEST

```
core/qrd-core/src/
├── schema/mod.rs          # Schema builder, field types, SHA-256 fingerprint
├── writer/mod.rs          # FileWriter, WriterConfig
├── writer/streaming_writer.rs  # StreamingWriter, bounded memory
├── writer/buffer_pool.rs  # BufferPool
├── reader/mod.rs          # FileReader, mmap vs in-memory threshold (64MB)
├── reader/partial_reader.rs    # PartialReader, column projection
├── reader/range_reader.rs      # RangeReader, HTTP byte-range reads
├── encoding/mod.rs        # Encoding selection, EncodingType enum
├── encoding/plain.rs      # PlainEncoder
├── encoding/rle.rs        # RleEncoder
├── encoding/bit_packed.rs # BitPackedEncoder
├── encoding/delta_binary.rs    # DeltaBinaryEncoder
├── encoding/delta_byte_array.rs # DeltaByteArrayEncoder
├── encoding/byte_stream_split.rs # ByteStreamSplitEncoder
├── encoding/dictionary_rle.rs   # DictionaryRleEncoder
├── compression/mod.rs     # CompressionCodec, compress/decompress
├── compression/entropy.rs # CompressionSelector, EntropyCalculator
├── encryption/mod.rs      # EncryptionConfig, AES-256-GCM, HKDF, Argon2id
├── ecc/mod.rs             # EccConfig, EccCodec, Reed-Solomon
├── footer/mod.rs          # Footer
├── footer/builder.rs      # FooterBuilder
├── footer/parser.rs       # FooterParser
├── columnar/mod.rs        # RowBuffer, row-to-column transposition
├── rowgroup/mod.rs        # RowGroup
├── metadata/column_stats.rs    # ColumnStats, FilterResult, QueryOptimizer
├── metadata/mod.rs        # MetadataIndex
├── validation/mod.rs      # Validator
├── validation/corruption.rs    # CorruptionDetector, CorruptionType
├── io/mod.rs              # BufferedReader, BufferedWriter
├── utils/bit_ops.rs       # BitOps
├── utils/simd.rs          # SimdOps (currently dead code)
├── utils/mod.rs           # varint encode/decode, write_header
└── error.rs               # Error enum (20+ variants)
```

## BINARY FORMAT CONSTANTS TO TEST AGAINST

```rust
const QRD_MAGIC: &[u8; 4] = b"QRD\x01";
const QRD_VERSION_MAJOR: u16 = 1;
const QRD_VERSION_MINOR: u16 = 0;
const DEFAULT_ROW_GROUP_SIZE: u32 = 100_000;
const DEFAULT_BUFFER_SIZE: usize = 8 * 1024 * 1024; // 8MB
const MMAP_THRESHOLD: u64 = 64 * 1024 * 1024; // 64MB
// File header is exactly 32 bytes
// Footer: last 4 bytes = FOOTER_LENGTH (u32 LE)
// Footer CRC32 = last 4 bytes of footer content
// Column chunk header: ENCODING_ID(1) + COMPRESSION_ID(1) + UNCOMPRESSED_LEN(4) + COMPRESSED_LEN(4) + NULL_COUNT(4) + DISTINCT_COUNT(4) = 18 bytes before payload
```

## TEST DISTRIBUTION — EXACTLY 600 TESTS

Distribute across these test files. Create each file fresh with all required imports. Every test must compile with `cargo test --package qrd-core`.

### File 1: tests/schema_comprehensive_test.rs — 60 tests
Cover:
- SchemaBuilder: empty schema, 1 field, 256 fields, 257 fields (if there's a limit)
- All FieldType variants: Int8, Int16, Int32, Int64, UInt8, UInt16, UInt32, UInt64, Float32, Float64, Boolean, Timestamp, Date, Time, Duration, Utf8String, Enum, Uuid, Blob, Decimal
- All Nullability variants: Required, Optional, Repeated
- Duplicate field name rejection
- Empty field name rejection
- Field name with special characters: spaces, unicode, dots, underscores, hyphens, slashes
- Field name at max length (255 chars)
- Field name exceeding max length
- Schema serialization/deserialization roundtrip
- Schema ID determinism: same schema → same ID, every time
- Schema ID uniqueness: different schemas → different IDs (test 20 pairs)
- Schema ID collision test with 4-byte vs 8-byte hash
- Schema with all optional fields
- Schema with all required fields
- Schema with mixed nullability
- Schema field ordering matters for ID
- Schema field metadata key/value pairs
- Schema metadata with empty value
- Schema metadata with unicode value
- Schema metadata key deduplication behavior
- Schema fingerprint changes when field type changes
- Schema fingerprint changes when field name changes
- Schema fingerprint changes when nullability changes
- Schema fingerprint changes when field order changes
- Schema clone equality
- Schema serialized byte length is deterministic
- Schema with 0 fields build() should error
- Schema builder chaining API

### File 2: tests/writer_comprehensive_test.rs — 80 tests
Cover:
- FileWriter::new creates valid file with correct header
- Header magic bytes at offset 0 are exactly b"QRD\x01"
- Header VERSION_MAJOR at offset 4 is 1u16 LE
- Header VERSION_MINOR at offset 6 is 0u16 LE
- Header SCHEMA_ID at offset 8 matches schema fingerprint
- Header COLUMN_COUNT at offset 20 matches schema field count
- Header ROW_GROUP_SIZE at offset 24 matches config
- Header FLAGS at offset 28 is 0u32 (reserved)
- Header is exactly 32 bytes
- finish() writes footer and footer_length
- finish() without any rows written
- finish() called twice should error or be idempotent
- write_row() with correct column count
- write_row() with too few columns (should error)
- write_row() with too many columns (should error)
- write_row() after finish() should error
- WriterConfig::default() has expected values (row_group_size=100_000, compression_level=3)
- Custom row_group_size=1 (one row per group)
- Custom row_group_size=1_000_000
- write_row 1 row then finish — single row group
- write_row exactly row_group_size rows — single row group, exact boundary
- write_row row_group_size + 1 rows — creates second row group
- write_row row_group_size * 3 rows — three row groups
- Row count in header matches total rows written
- Row group offsets in footer are monotonically increasing
- Row group offsets in footer are correct (verify by seeking)
- All-null row (all Optional fields with empty Vec)
- Alternating null/non-null pattern
- Row with maximum string length (1MB)
- Row with empty blob
- Row with 1-byte blob
- Row with 10MB blob
- Boolean field: write 0x00 and 0x01
- Boolean field: write invalid byte (e.g., 0x02) — behavior defined?
- Int8 min/max boundaries: -128, 127
- Int16 min/max boundaries
- Int32 min/max boundaries
- Int64 min/max: i64::MIN, i64::MAX
- UInt8 min/max: 0, 255
- UInt64 max: u64::MAX
- Float32: NaN, +Infinity, -Infinity, 0.0, -0.0, MIN_POSITIVE, MAX
- Float64: NaN, +Infinity, -Infinity, 0.0, -0.0, MIN_POSITIVE, MAX
- Timestamp: epoch (0), max i64, negative timestamp
- UUID: 16-byte raw, all zeros, all 0xFF
- Decimal: empty, single byte, 100-byte payload
- String with null bytes embedded (\x00 in middle)
- String with only null bytes
- String with emoji and full Unicode (multi-byte UTF-8)
- String with BOM (U+FEFF)
- String with RTL characters
- Enum field: valid index, index at dictionary boundary
- CRC32 of each column chunk is written correctly (verify by parsing)
- Footer CRC32 is correct (verify by parsing)
- Concurrent writes to different files (no shared state)
- WriterConfig with encryption=None, ecc=None (default)
- WriterConfig with encryption enabled
- WriterConfig with ecc enabled
- WriterConfig with both encryption and ecc
- WriterConfig with per_column_encryption=true
- WriterConfig with encrypt_footer=false
- WriterConfig with compress_level 0 through 10
- Output file size grows monotonically with more rows
- write_rows (batch) vs write_row (individual) produce identical output
- BufferPool initialization and release
- BufferPool reuse after flush

### File 3: tests/reader_comprehensive_test.rs — 80 tests
Cover:
- FileReader::new on non-existent path should error
- FileReader::new on empty file should error
- FileReader::new on file with only 4 bytes should error
- FileReader::new on file with invalid magic should error (InvalidMagic)
- FileReader::new on file with valid magic but wrong version should error (UnsupportedVersion)
- FileReader::new on file with footer_length > file_size should error
- FileReader::new on file with footer_length = 0
- FileReader::new on file with correct header but truncated footer
- FileReader::new selects mmap for file >= 64MB
- FileReader::new selects in-memory for file < 64MB
- FileReader::from_bytes with valid data
- FileReader::from_bytes with empty bytes should error
- FileReader::from_slice with valid data
- FileReader::open_mmap explicitly
- FileReader::open_in_memory explicitly
- reader.schema() returns correct schema after write/read
- reader.row_count() returns 0 for empty file
- reader.row_count() returns correct count for 1 row
- reader.row_count() for 100_000 rows (one full group)
- reader.row_count() for 100_001 rows (two groups)
- reader.row_count() for 1_000_000 rows (ten groups)
- reader.row_group_offsets() is non-empty for non-empty file
- reader.row_group_offsets() length matches number of groups written
- reader.rows() iterates all rows in order
- reader.rows() on empty file returns empty iterator
- Row values roundtrip: Int32 written then read matches exactly
- Row values roundtrip: Int64
- Row values roundtrip: Float32 (bitwise equality)
- Row values roundtrip: Float64
- Row values roundtrip: Boolean
- Row values roundtrip: String (UTF-8)
- Row values roundtrip: Blob (binary)
- Row values roundtrip: Null value (Optional field, empty)
- Row values roundtrip: UUID (16 bytes)
- Row values roundtrip: Timestamp
- CRC32 validation triggers error on corrupted chunk (flip one bit in payload area)
- Footer CRC32 validation triggers error when footer bytes corrupted
- Partial read: read only column 0 from multi-column file
- Partial read: read only last column
- Partial read: read columns [0, 2] skipping column 1
- Partial read: read all columns equals full read
- Partial read: request non-existent column index should error
- Partial read: empty column index list behavior
- PartialReadConfig construction
- PartialReader on single-column file
- RangeReader: construct with valid byte ranges
- RangeReader: range [0, 32) returns header
- RangeReader: read beyond file size should error
- RangeReader: overlapping ranges
- Footer-only inspection: schema available without reading payload
- Footer-only inspection: row_count available without reading payload
- Footer-only inspection: column statistics available without reading payload
- Multi-file: same schema, different data — schema IDs match
- Multi-file: different schemas — schema IDs differ
- FileReader on ZSTD-compressed file
- FileReader on LZ4-compressed file
- FileReader on uncompressed file
- FileReader on file with mixed compression per column
- FileReader with encryption config — decrypt successfully
- FileReader with wrong encryption key — error
- FileReader with no encryption config on encrypted file — error
- FileReader with ECC on file without ECC
- FileReader on file with all-null columns
- FileReader handles floating point special values (NaN, Inf) round-trip

### File 4: tests/encoding_comprehensive_test.rs — 80 tests
Cover:
- EncodingType::from_id(0) through from_id(7) all succeed
- EncodingType::from_id(8) should error
- EncodingType::from_id(255) should error
- EncodingType::to_id() and from_id() are inverse for all 8 types
- All 8 EncodingType display strings match spec: PLAIN, RLE, BIT_PACKED, DELTA_BINARY, DELTA_BYTE_ARRAY, BYTE_STREAM_SPLIT, DICTIONARY_RLE, PASSTHROUGH
- PlainEncoder: encode empty data → empty output
- PlainEncoder: encode/decode [1,2,3,4,5] exact roundtrip
- PlainEncoder: encode/decode 1MB of random data
- PlainEncoder: decode with wrong expected_len — behavior defined?
- PlainEncoder: determinism — same input → same output twice
- RleEncoder: all same values (run of 1M identical bytes)
- RleEncoder: no runs (all unique values)
- RleEncoder: alternating values [A, B, A, B, ...]
- RleEncoder: single value
- RleEncoder: empty input
- RleEncoder: encode/decode roundtrip with 10_000 elements
- RleEncoder: encode/decode roundtrip: strings
- BitPackedEncoder: encode 8 booleans into 1 byte
- BitPackedEncoder: encode 0 booleans
- BitPackedEncoder: encode 1 boolean (true)
- BitPackedEncoder: encode 1 boolean (false)
- BitPackedEncoder: encode 7 booleans (partial byte)
- BitPackedEncoder: encode 9 booleans (1 byte + 1 partial)
- BitPackedEncoder: encode 1024 alternating true/false
- BitPackedEncoder: decode with wrong expected_len
- BitPackedEncoder: all-true 1000 values
- BitPackedEncoder: all-false 1000 values
- DeltaBinaryEncoder: monotonic sequence [0,1,2,...,999]
- DeltaBinaryEncoder: reverse sequence [999,...,1,0]
- DeltaBinaryEncoder: constant sequence [42, 42, 42, ...]
- DeltaBinaryEncoder: timestamp-like sequence (microsecond increments)
- DeltaBinaryEncoder: large deltas (i64 boundaries)
- DeltaBinaryEncoder: negative deltas
- DeltaBinaryEncoder: encode/decode 100_000 element sequence
- DeltaBinaryEncoder: empty input
- DeltaBinaryEncoder: single element
- DeltaBinaryEncoder: data length not multiple of 8 should error
- DeltaByteArrayEncoder: common prefix strings ["http://a.com", "http://b.com"]
- DeltaByteArrayEncoder: no common prefix
- DeltaByteArrayEncoder: identical strings
- DeltaByteArrayEncoder: empty strings
- DeltaByteArrayEncoder: single string
- DeltaByteArrayEncoder: strings with embedded nulls
- DeltaByteArrayEncoder: very long strings (100KB each)
- DeltaByteArrayEncoder: encode/decode 10_000 URL-like strings
- ByteStreamSplitEncoder: float32 array roundtrip (data.len() % 4 == 0)
- ByteStreamSplitEncoder: float64 array roundtrip (data.len() % 8 == 0)
- ByteStreamSplitEncoder: data length not multiple of 4 or 8 — error path
- ByteStreamSplitEncoder: single float32
- ByteStreamSplitEncoder: 1000 float32 values
- ByteStreamSplitEncoder: NaN and Inf values preserved bitwise
- ByteStreamSplitEncoder: all-zero floats
- DictionaryRleEncoder: low cardinality strings (5 unique values, 10K elements)
- DictionaryRleEncoder: single unique value repeated 1M times
- DictionaryRleEncoder: all unique values (cardinality = length)
- DictionaryRleEncoder: empty data
- DictionaryRleEncoder: encode/decode roundtrip preserves order
- DictionaryRleEncoder: integer dictionary (Int64 column)
- select_encoding() for Int64 monotonic → DeltaBinary
- select_encoding() for String low-cardinality → DictionaryRle
- select_encoding() for Boolean → BitPacked
- select_encoding() for Float64 → ByteStreamSplit
- select_encoding() for empty data → does not panic
- select_encoding() is deterministic: same call twice → same result
- is_low_cardinality() with 5 unique values, threshold 10 → true
- is_low_cardinality() with 15 unique values, threshold 10 → false
- is_low_cardinality_integers() with 3 unique i64 → true
- is_low_cardinality_integers() with invalid data length (not multiple of 8) → false
- All encoders: encode then decode → original data (property: roundtrip)
- All encoders: encode output is non-empty for non-empty input (except special cases)
- All encoders: encode is deterministic

### File 5: tests/compression_comprehensive_test.rs — 50 tests
Cover:
- CompressionCodec::from_id(0) → None
- CompressionCodec::from_id(1) → Zstd
- CompressionCodec::from_id(2) → Lz4
- CompressionCodec::from_id(3) → None (GZIP reserved, not implemented)
- CompressionCodec::from_id(255) → None (not Some)
- CompressionCodec display: "NONE", "ZSTD", "LZ4"
- CompressionLevel::new(0) through new(10) all valid
- CompressionLevel::new(11) clamps to 10
- CompressionLevel::new(255) clamps to 10
- compress(empty, None) → empty
- compress(empty, Zstd) → valid (decompresses to empty)
- compress(empty, Lz4) → valid
- compress/decompress roundtrip: None (identity)
- compress/decompress roundtrip: Zstd level 0
- compress/decompress roundtrip: Zstd level 3 (default)
- compress/decompress roundtrip: Zstd level 10
- compress/decompress roundtrip: Lz4
- compress: highly repetitive data (10MB zeros) — Zstd ratio < 0.01
- compress: highly repetitive data — Lz4 ratio < 0.1
- compress: random data — Zstd output size <= input * 1.1 (shouldn't expand much)
- decompress: truncated Zstd data → error
- decompress: truncated Lz4 data → error
- decompress: corrupted Zstd (flip middle byte) → error
- decompress: empty input with Zstd → error
- decompress: empty input with Lz4 → error
- decompress: Lz4-compressed data with Zstd → error
- decompress: Zstd-compressed data with Lz4 → error
- decompress: None codec on any data → identity (no error)
- EntropyCalculator: constant data → entropy = 0.0
- EntropyCalculator: uniform random data → entropy ≈ 8.0
- EntropyCalculator: bimodal data → entropy ≈ 1.0
- EntropyCalculator: empty data → 0.0 (no panic)
- EntropyCalculator: single byte → entropy = 0.0
- CompressionSelector: low entropy data → recommends Zstd
- CompressionSelector: high entropy data → recommends None or Lz4
- CompressionSelector: streaming workload → recommends Lz4
- Chunk-level independence: decompress column 0 without touching column 1 data
- Compression at each level produces valid decompressible output
- Large data (100MB): compress and decompress without OOM
- Zstd with level 1 vs level 10: level 10 output <= level 1 output for repetitive data
- CompressionCodec::to_id() and from_id() roundtrip for all 3 codecs
- compress then decompress 1 million rows of realistic telemetry data
- CompressionLevel default is 3
- Parallel decompression: 8 independent chunks can be decompressed concurrently

### File 6: tests/encryption_fixed_test.rs — 60 tests
(This file REPLACES encryption_edge_cases_test.rs with all compile errors fixed)
Cover:
- EncryptionConfig::new(vec![0u8; 32]) → Ok
- EncryptionConfig::new(vec![0u8; 31]) → Err (wrong key length)
- EncryptionConfig::new(vec![0u8; 33]) → Err
- EncryptionConfig::new(vec![]) → Err
- EncryptionConfig::generate_key() returns 32 bytes
- EncryptionConfig::generate_key() called twice returns different keys (randomness)
- EncryptionConfig::generate_salt() returns 32 bytes
- EncryptionConfig::with_salt(valid_key, valid_salt) → Ok
- EncryptionConfig::with_salt(valid_key, wrong_salt_length) → Err
- EncryptionConfig::derive_from_password("password", &salt) → Ok, returns 32-byte key config
- EncryptionConfig::derive_from_user_password("password", None) → Ok (generates salt)
- EncryptionConfig::derive_from_user_password("password", Some(&salt)) → Ok
- Same password + same salt → same derived key (deterministic)
- Different passwords → different derived keys
- Same password + different salt → different derived keys
- encrypt(data, config) → ciphertext != plaintext
- decrypt(encrypt(data, config), config) → data (roundtrip)
- decrypt with wrong key → error
- encrypt empty data → some output (at minimum nonce + auth_tag = 28 bytes)
- encrypt 1 byte
- encrypt 1MB
- encrypt 10MB
- encrypt/decrypt: all-zero key
- encrypt/decrypt: all-0xFF key
- encrypt/decrypt: NaN floats in data
- encrypt/decrypt: null bytes in data
- Nonce uniqueness: encrypt same data twice → different ciphertexts (nonce is random)
- Auth tag: modify 1 bit in ciphertext → decrypt error (tamper detection)
- Auth tag: modify 1 bit in nonce → decrypt error
- Per-column key derivation: derive_column_key("col1") → 32 bytes
- Per-column key derivation: derive_column_key("col1") is deterministic
- Per-column key derivation: derive_column_key("col1") != derive_column_key("col2")
- Per-column key derivation: different master key → different column key
- File write/read with encryption: full pipeline test
- File with encryption: encrypted file bytes do not contain plaintext strings
- File with per_column_encryption=true, encrypt_footer=false
- File with per_column_encryption=true, encrypt_footer=true
- File with encryption + compression: ZSTD then encrypt
- File with encryption + compression: LZ4 then encrypt
- Multiple row groups: each group independently decryptable
- EncryptionConfig clone is identical
- EncryptionConfig with all-zeros key encrypts and decrypts
- Argon2id: hash is slow (takes > 10ms, proving work factor is non-trivial)
- HKDF expansion: same IKM + same info → same output
- HKDF expansion: different info → different output
- WriterConfig with encryption enabled, file created and readable
- WriterConfig with encryption disabled, file created and readable
- Reader with encryption key reads encrypted file
- Reader without encryption key returns error on encrypted file
- Reader with wrong encryption key returns error
- Encryption key stored nowhere in plaintext in output file

### File 7: tests/ecc_comprehensive_test.rs — 40 tests
Cover:
- EccConfig::new(1) → Ok (1 parity chunk)
- EccConfig::new(32) → Ok (32 parity chunks, max)
- EccConfig::new(0) → Err (0 not allowed)
- EccConfig::new(33) → Err (exceeds max)
- EccConfig::with_chunk_size(2, 4096) → Ok
- EccConfig::with_chunk_size(2, 0) → Err
- EccConfig::with_chunk_size(2, 65536) → Ok
- EccConfig::with_chunk_size(2, 65537) → Err
- EccConfig::total_shards(data_shards=4) for parity=2 → 6
- EccConfig::max_data_shards(total_shards=6) for parity=2 → 4
- EccCodec::encode small data (< chunk_size)
- EccCodec::encode data exactly = chunk_size
- EccCodec::encode data = 2 * chunk_size
- EccCodec::encode empty data
- EccCodec::encode/decode roundtrip: data recovered intact (no corruption)
- EccCodec::encode produces data_shards + parity_shards total shards
- EccEncodedData: mark 0 shards as lost → decode succeeds
- EccEncodedData: mark 1 shard as lost (parity=2) → decode recovers
- EccEncodedData: mark 2 shards as lost (parity=2) → decode recovers
- EccEncodedData: mark 3 shards as lost (parity=2) → decode fails (exceeds parity)
- Corruption recovery: flip bytes in 1 data shard → decode recovers
- Corruption recovery: corrupt parity shard → decode still works
- ECC with encryption: encode then encrypt, decrypt then decode
- ECC with compression: encode then compress, decompress then decode
- File write with ECC config: creates valid file
- File read with ECC config: reads back all data
- Multiple row groups with ECC: each group has independent parity
- ECC with large blob (10MB): encode/decode
- ECC parity_chunks=1: minimum protection
- ECC parity_chunks=16: high protection
- ECC data integrity: parity chunks can reconstruct any single lost shard
- EccConfig clone is identical
- Reed-Solomon: correct GF(2^8) arithmetic (spot check: encode [1,0] with 1 parity)
- ECC overhead: output_size = input_size * (1 + parity_chunks / data_shards) approximately
- ECC with mixed null/non-null data
- ECC with highly compressible data
- ECC with random high-entropy data
- Writer with ecc=Some(...): file structure includes parity chunks
- ECC encode then corrupt → detect corruption via CRC mismatch
- ECC minimum chunk_size=1 (boundary test)

### File 8: tests/footer_comprehensive_test.rs — 50 tests
Cover:
- Footer::new(schema, row_count) initializes correctly
- Footer::new with row_count=0
- Footer::new with row_count=u32::MAX
- Footer checksum is 0 before serialization
- Footer::serialize() produces non-empty bytes
- Footer::deserialize(serialize(footer)) → identical footer (roundtrip)
- Footer serialized bytes contain valid CRC32 at the end
- Footer CRC32 verification: flip 1 bit → parse error
- Footer CRC32 verification: flip last byte of CRC → error
- Footer with 0 row groups
- Footer with 1 row group offset
- Footer with 1000 row group offsets
- Footer row group offsets are sorted ascending
- Footer schema version is written and read correctly
- Footer schema field count matches schema
- Footer schema fields serialized in order
- Footer schema field: name preserved exactly (including unicode)
- Footer schema field: field_type preserved exactly
- Footer schema field: nullability preserved exactly
- Footer schema field: metadata entries preserved
- Footer statistics_flag: 0 means no stats, 1 means stats present
- Footer with statistics: min/max/null_count per column preserved
- Footer metadata_length and metadata_bytes roundtrip
- FooterBuilder: build from writer state
- FooterBuilder: adding row group offsets incrementally
- FooterParser::parse on valid bytes → Ok
- FooterParser::parse on empty bytes → Err
- FooterParser::parse on 3 bytes → Err (too small for CRC32)
- FooterParser::parse where stated length > actual bytes → Err
- FooterParser::parse where CRC32 is wrong → Err
- FooterParser: version field parsed correctly
- FooterParser: schema_length field parsed correctly
- FooterParser: row_group_count parsed correctly
- FooterParser: row_group_offsets array length matches row_group_count
- File footer is at exactly file_size - 4 - FOOTER_LENGTH bytes
- FOOTER_LENGTH field is at exactly file_size - 4 offset
- FOOTER_LENGTH field is u32 little-endian
- Footer parsing protocol: seek to end, read 4 bytes, seek back, parse
- Footer with 0 statistics (statistics_flag = 0)
- Footer with empty metadata (metadata_length = 0)
- Footer with large metadata (1KB metadata blob)
- Footer schema ID matches schema fingerprint
- Footer row_group_offsets all > 32 (must come after header)
- Footer row_group_offsets none overlap (no two groups at same offset)
- Footer after multiple write sessions: offsets accumulate correctly
- Footer maximum field count schema: 256 fields
- Footer serialization is deterministic (same footer → same bytes)
- Footer schema field name with max length (255 chars) roundtrips
- Footer with unknown optional fields: reader should ignore safely

### File 9: tests/validation_comprehensive_test.rs — 40 tests
Cover:
- Validator::validate_magic(b"QRD\x01") → Ok
- Validator::validate_magic(b"QRD\x00") → Err (wrong version byte)
- Validator::validate_magic(b"qrd\x01") → Err (wrong case)
- Validator::validate_magic(b"") → Err
- Validator::validate_magic(b"QRD") → Err (too short)
- Validator::validate_magic(b"QRD\x01X") → behavior defined?
- Validator::validate_version(1, 0) → Ok
- Validator::validate_version(0, 0) → Err
- Validator::validate_version(2, 0) → Err (future major)
- Validator::validate_version(1, 1) → Ok (future minor, should be accepted)
- Validator::validate_schema(valid_schema) → Ok
- Validator::validate_schema(schema with 0 fields) → Err
- Validator::validate_schema(schema with duplicate field names) → Err
- Validator::validate_row(row matching schema) → Ok
- Validator::validate_row(row with wrong column count) → Err
- Validator::validate_row(Required field with empty value) → Err
- Validator::validate_row(Optional field with empty value) → Ok (null is valid)
- CorruptionDetector::check_crc32(data, expected_crc) → Ok when correct
- CorruptionDetector::check_crc32(data, wrong_crc) → Err(CrcMismatch)
- CorruptionDetector::check_crc32(empty_data, crc32_of_empty) → Ok
- CorruptionDetector: detect bit flip in first byte
- CorruptionDetector: detect bit flip in last byte
- CorruptionDetector: detect bit flip in middle
- CorruptionDetector: single bit flip detection (check 8 positions in 1 byte)
- CorruptionType variants: all variants can be created and displayed
- CorruptionDetector on column chunk: validates header + payload
- CorruptionDetector on footer: validates footer CRC
- Validator on truncated file (< 36 bytes): rejects
- Validator on file with correct header but zero-length footer
- Validator on file with FOOTER_LENGTH pointing before start of data
- Validator on file with footer_length = file_size - 4 (footer is whole file)
- Validator: full file validation pipeline on known-good file → Ok
- Validator: full file validation pipeline on 1-byte-corrupted file → Err
- Validator: schema_id in header matches schema in footer
- Validator: row_count in header matches sum of row_groups in footer
- Validator: column_count in header matches schema field count in footer
- Validator: all row_group_offsets are within file bounds
- Validator: no row_group_offsets overlap each other
- Validator: reject file with footer_length > 1MB (protection against malicious input)

### File 10: tests/io_comprehensive_test.rs — 30 tests
Cover:
- BufferedReader::new with buffer_size=1
- BufferedReader::new with buffer_size=1MB
- BufferedReader::read_exact_bytes(0) → Ok(empty)
- BufferedReader::read_exact_bytes(1) → Ok([first_byte])
- BufferedReader::read_exact_bytes(n) where n > file size → Ok(fewer bytes or Err)
- BufferedReader::read_exact_bytes across buffer boundary (buffer_size=4, read 6 bytes)
- BufferedReader::seek(0) → position is 0
- BufferedReader::seek(n) → position is n
- BufferedReader::seek beyond end → Ok(position=beyond_end)
- BufferedReader::position() after no reads → 0
- BufferedReader::position() after read → correct offset
- BufferedReader::read then seek back then read again → same bytes
- BufferedWriter::new with buffer_size=1
- BufferedWriter::new with buffer_size=1MB
- BufferedWriter::write_bytes([]) → Ok, buffer unchanged
- BufferedWriter::write_bytes([1,2,3]) → buffer has 3 bytes
- BufferedWriter::write_bytes fills buffer → auto-flush
- BufferedWriter::flush() → buffer empty, inner has bytes
- BufferedWriter::flush() on empty buffer → Ok
- BufferedWriter::finish() → inner writer has all bytes
- BufferedWriter::finish() then write_bytes → Err or panic
- write_header() produces exactly 32 bytes
- write_header() magic bytes at offset 0 are b"QRD\x01"
- write_header() version_major at offset 4 is 1u16 LE
- write_header() version_minor at offset 6 is 0u16 LE
- varint::encode(0) → [0]
- varint::encode(127) → [127]
- varint::encode(128) → [0x80, 0x01]
- varint::encode(u64::MAX) → 10 bytes
- varint encode/decode roundtrip for 0, 1, 127, 128, 255, 1024, 65535, u32::MAX as u64, u64::MAX

### File 11: tests/columnar_comprehensive_test.rs — 30 tests
Cover:
- RowBuffer::new(0 columns) behavior
- RowBuffer::new(1 column)
- RowBuffer::new(256 columns)
- RowBuffer: push 1 row, verify column data
- RowBuffer: push N rows, verify row count
- RowBuffer: column data is stored in column-major order
- RowBuffer: transpose: 3 rows × 3 cols → 3 column vectors of 3 elements
- RowBuffer: transpose empty buffer → empty columns
- RowBuffer: push to buffer until flush threshold → auto-flush behavior
- RowBuffer: mixed types per column (each column has homogeneous data)
- RowBuffer: null handling — Optional column with empty Vec
- RowBuffer: Required column with data
- RowBuffer: all-null column (all rows have empty Vec for Optional column)
- RowGroup: construct from column data
- RowGroup: serialize to bytes and deserialize back
- RowGroup: column chunk headers are present for each column
- RowGroup: row group header has correct row count
- RowGroup: row group footer has CRC32
- RowGroup: CRC32 calculated over column payload
- RowGroup: add ECC parity after column chunks
- RowGroup: position in file matches recorded offset in footer
- RowGroup: column chunk ENCODING_ID matches selected encoder
- RowGroup: column chunk COMPRESSION_ID matches selected compressor
- RowGroup: UNCOMPRESSED_LEN matches actual uncompressed size
- RowGroup: COMPRESSED_LEN matches actual compressed size
- RowGroup: NULL_COUNT matches actual null values in column
- RowGroup: DISTINCT_COUNT matches actual unique values
- Column transposition correctness: row [1,A,X], [2,B,Y], [3,C,Z] → col0=[1,2,3], col1=[A,B,C], col2=[X,Y,Z]
- Column transposition with nulls: row [1,null,X] → col1 contains null
- RowBuffer max row count before flush is DEFAULT_ROW_GROUP_SIZE

### File 12: tests/metadata_comprehensive_test.rs — 30 tests
Cover:
- ColumnStats::new initializes with zero counts
- ColumnStats::update(Some(&value)) increments total_count
- ColumnStats::update(None) increments null_count and total_count
- ColumnStats::update: min_value set on first non-null
- ColumnStats::update: max_value set on first non-null
- ColumnStats::update: min_value updates when smaller value seen
- ColumnStats::update: max_value updates when larger value seen
- ColumnStats::update: min/max unchanged for null values
- ColumnStats::distinct_count increments for new unique values
- ColumnStats::distinct_count does NOT increment for repeated values
- ColumnStats: update 1000 times with same value → distinct_count = 1
- ColumnStats: can_pass_filter IsNull on column with nulls → MayPass
- ColumnStats: can_pass_filter IsNull on column with no nulls → MustNotPass
- ColumnStats: can_pass_filter IsNotNull on column with all nulls → MustNotPass
- ColumnStats: can_pass_filter Equal(v) where v in [min,max] → MayPass
- ColumnStats: can_pass_filter Equal(v) where v > max → MustNotPass
- ColumnStats: can_pass_filter Equal(v) where v < min → MustNotPass
- ColumnStats: can_pass_filter GreaterThan(max) → MustNotPass
- ColumnStats: can_pass_filter LessThan(min) → MustNotPass
- ColumnStats: can_pass_filter Between(lo, hi) where hi < min → MustNotPass
- FilterResult::MayPass.combine(MayPass) → MayPass
- FilterResult::MayPass.combine(MustNotPass) → MustNotPass
- FilterResult::MustNotPass.combine(MustNotPass) → MustNotPass
- RowGroupStats: tracks per-column stats
- RowGroupStats: can_pass_filters with multi-column filter
- MetadataIndex::new constructs correctly
- MetadataIndex::get_column_index by name
- MetadataIndex::get_column_index for non-existent column → None
- QueryOptimizer::optimize_access filters out non-matching row groups
- MetadataIndex serialization/deserialization roundtrip (serde_json)

## CODE QUALITY REQUIREMENTS

Every single test must:

1. **Compile without warnings** under `cargo clippy -- -D warnings`
2. **Use correct API types** — never pass `[u8; 32]` where `Vec<u8>` is expected; use `.to_vec()` explicitly
3. **Not use `FieldType::Bool`** — use `FieldType::Boolean` if it exists, or `FieldType::Int8` as substitute
4. **Not use `assert!(true)`** — every assertion must test something real
5. **Use `tempfile::NamedTempFile`** for all file-based tests
6. **Import only what is used** — no unused imports
7. **Use `.div_ceil(n)` instead of `(x + n - 1) / n`**
8. **Use `.is_multiple_of(n)` instead of `x % n == 0`**
9. **Use array/slice literals** instead of `vec![...]` where the type allows
10. **Implement `Default`** or use struct update syntax where needed
11. **No `clone()` on Copy types** — use the value directly
12. **Document `unsafe` blocks** with `// SAFETY:` comments

## TEST NAMING CONVENTION

```rust
#[test]
fn test_<module>_<scenario>_<expected_outcome>() {
    // arrange
    // act
    // assert
}
```

Examples:
- `test_schema_empty_name_returns_error`
- `test_writer_finish_without_rows_creates_valid_file`
- `test_reader_mmap_threshold_64mb_uses_mmap`
- `test_encryption_wrong_key_returns_decrypt_error`
- `test_footer_crc32_flip_returns_error`

## HELPER FUNCTIONS

Define these helpers at the top of each file to avoid duplication:

```rust
// Create a minimal valid schema
fn minimal_schema() -> Schema { ... }

// Create a schema with all field types
fn full_schema() -> Schema { ... }

// Write N rows to a temp file and return the path
fn write_n_rows(n: usize, schema: Schema) -> NamedTempFile { ... }

// Flip bit at position pos in a Vec<u8>
fn flip_bit(data: &mut Vec<u8>, byte_pos: usize, bit_pos: u8) { ... }

// Create a 32-byte encryption key
fn test_key() -> Vec<u8> { vec![0x42u8; 32] }

// Assert file header magic is correct
fn assert_valid_header(path: &std::path::Path) { ... }
```

## OUTPUT FORMAT

Generate each test file completely. For each file:

1. Start with the file path as a comment: `// tests/schema_comprehensive_test.rs`
2. All `use` statements
3. Helper functions
4. All test functions, each with `#[test]` attribute
5. Count tests at the end as a comment: `// Total: N tests`

Verify the count before outputting. The total across all 12 files must be exactly 600.

## PRIORITY ORDER

Generate files in this order (most critical first):
1. encryption_fixed_test.rs (fixes active compile failures)
2. footer_comprehensive_test.rs (format integrity)
3. writer_comprehensive_test.rs (core functionality)
4. reader_comprehensive_test.rs (core functionality)
5. validation_comprehensive_test.rs (security)
6. schema_comprehensive_test.rs
7. encoding_comprehensive_test.rs
8. compression_comprehensive_test.rs
9. ecc_comprehensive_test.rs
10. io_comprehensive_test.rs
11. columnar_comprehensive_test.rs
12. metadata_comprehensive_test.rs

Begin with File 1 (encryption_fixed_test.rs) now.
```
