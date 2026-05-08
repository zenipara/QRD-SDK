# Encoding Specification

**Document Version:** 1.0.0  
**Status:** Draft

## Overview

This document specifies all encoding algorithms available in QRD.

## Encoding Selection Algorithm

Automatic encoding selection based on column properties:

```rust
fn select_encoding(
    logical_type: LogicalType,
    column_data: &[Value],
    config: &EncodingConfig,
) -> Encoding {
    match logical_type {
        LogicalType::Boolean => Encoding::BitPacked,
        LogicalType::Blob => Encoding::Passthrough,
        LogicalType::String => select_string_encoding(column_data),
        _ if is_sorted(column_data) => Encoding::DeltaBinary,
        _ if is_highly_repetitive(column_data) => Encoding::Rle,
        _ => Encoding::Plain,
    }
}
```

## 1. PLAIN Encoding

Raw sequential bytes. Used when no encoding benefit exists.

**When:**
- Mixed data patterns
- Already encoded blobs
- Default fallback

**Format:**
```
For required columns:
  [value_1] [value_2] ... [value_n]

For optional columns:
  [null_bitmap] [value_1] [value_2] ...
```

**Space:** O(n) where n = column size

---

## 2. RLE (Run-Length Encoding)

Compresses repetitive values.

**When:**
- Low cardinality (< 10 distinct values)
- Repetitive patterns
- Boolean columns with long runs

**Format:**
```
[run_length: U32LE] [value: bytes] [run_length: U32LE] [value: bytes] ...

Example (INT32):
  Value sequence: [5, 5, 5, 5, 5, 3, 3, 1]
  Encoded as:
    run_len=5, value=5 → [0x05000000] [0x05000000]
    run_len=2, value=3 → [0x02000000] [0x03000000]
    run_len=1, value=1 → [0x01000000] [0x01000000]
```

**Algorithm:**
```rust
fn encode_rle(values: &[T]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < values.len() {
        let current = values[i];
        let mut run_length = 1;
        
        while i + run_length < values.len() 
            && values[i + run_length] == current {
            run_length += 1;
        }
        
        result.extend_from_slice(&(run_length as u32).to_le_bytes());
        result.extend_from_slice(&value_to_bytes(&current));
        
        i += run_length;
    }
    
    result
}
```

**Decoder:**
```rust
fn decode_rle(data: &[u8], row_count: u32) -> Vec<T> {
    let mut result = Vec::with_capacity(row_count as usize);
    let mut offset = 0;
    
    while offset + size_of::<T>() + 4 <= data.len() {
        let run_len = u32::from_le_bytes([
            data[offset], data[offset+1], 
            data[offset+2], data[offset+3]
        ]) as usize;
        offset += 4;
        
        let value = read_value::<T>(&data[offset..]);
        offset += size_of::<T>();
        
        for _ in 0..run_len {
            result.push(value);
        }
    }
    
    result
}
```

---

## 3. BIT_PACKED Encoding

Pack boolean and small integer bits.

**When:**
- Boolean columns
- INT8 with small range
- High-cardinality booleans

**Format:**
```
8 bits per byte, LSB first

Example (BOOLEAN):
  [true, false, true, true, false, false, true, false]
  Bits: 1, 0, 1, 1, 0, 0, 1, 0
  Byte: 0b10110100 = 0xB4
  
  Note: bit 0 (LSB) = first row
```

**Algorithm:**
```rust
fn encode_bit_packed(bools: &[bool]) -> Vec<u8> {
    let byte_count = (bools.len() + 7) / 8;
    let mut result = vec![0u8; byte_count];
    
    for (i, &is_true) in bools.iter().enumerate() {
        if is_true {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            result[byte_idx] |= 1 << bit_idx;
        }
    }
    
    result
}
```

**Space:** Floor(n/8) bytes for n booleans

---

## 4. DELTA_BINARY Encoding

Delta-of-deltas encoding for sorted integer data.

**When:**
- Sorted INTEGER columns
- Monotonic sequences
- Time series data

**Format:**
```
[min_value: value_bytes]
[first_delta: varint]
[block_size: U16LE]

For each block:
  [delta_block: variable-length encoded]

Example (INT32):
  Values: [100, 102, 105, 107, 110]
  Deltas:      [2,   3,   2,   3]

  min = 100
  Store min, then first_delta = 2
  
  Block: delta of deltas: [1, -1, 1]
  Encoded as varint
```

**Decoder:**
```rust
fn decode_delta_binary(data: &[u8], row_count: u32) -> Vec<i64> {
    let mut result = Vec::with_capacity(row_count as usize);
    
    let min_value = read_i64(&data[0..8]);
    result.push(min_value);
    
    let first_delta = varint_decode(&data[8..]);
    result.push(min_value + first_delta);
    
    let mut offset = 8 + varint_size(first_delta);
    let mut last_delta = first_delta;
    
    for _ in 2..row_count {
        let delta_of_delta = varint_decode(&data[offset..]);
        let new_delta = last_delta + delta_of_delta;
        result.push(result.last().unwrap() + new_delta);
        offset += varint_size(delta_of_delta);
        last_delta = new_delta;
    }
    
    result
}
```

**Space:** Depends on sequence monotonicity. Highly efficient for sorted data.

---

## 5. DELTA_BYTE_ARRAY Encoding

Delta encoding for variable-length byte arrays (strings).

**When:**
- Sorted UTF8_STRING columns
- Sorted BLOB columns
- Dictionary when combined with RLE

**Format:**
```
[len_prefix_encoding]
[first_value: bytes]

For each subsequent value:
  [shared_prefix_len: varint]
  [suffix: bytes]

Example (UTF8):
  ["apple", "apply", "apricot", "banana"]
  
  First: "apple"
  Second: shared_prefix=3 (app), suffix="ly"
  Third: shared_prefix=2 (ap), suffix="ricot"
  Fourth: shared_prefix=0, suffix="banana"
```

**Decoder:**
```rust
fn decode_delta_byte_array(data: &[u8], row_count: u32) -> Vec<Vec<u8>> {
    let mut result = Vec::with_capacity(row_count as usize);
    
    let mut offset = 0;
    let first_len = varint_decode(&data[offset..]);
    offset += varint_size(first_len);
    
    let mut prev = data[offset..offset+first_len].to_vec();
    result.push(prev.clone());
    offset += first_len;
    
    for _ in 1..row_count {
        let shared_len = varint_decode(&data[offset..]);
        offset += varint_size(shared_len);
        
        let suffix_len = varint_decode(&data[offset..]);
        offset += varint_size(suffix_len);
        
        let mut current = prev[0..shared_len].to_vec();
        current.extend_from_slice(&data[offset..offset+suffix_len]);
        offset += suffix_len;
        
        result.push(current.clone());
        prev = current;
    }
    
    result
}
```

---

## 6. BYTE_STREAM_SPLIT Encoding

Rearrange floating-point bytes for better compression.

**When:**
- FLOAT32 or FLOAT64 columns
- High precision numerical data
- Entropy-reducing pre-compression

**Format:**
```
[stream_0: all MSBs]
[stream_1: all second bytes]
[stream_2: all third bytes]
...
[stream_n: all LSBs]

Example (FLOAT32):
  Values: [1.0,    2.0,    3.0,   ]
  Bytes:  [00 00 80 3f, 00 00 00 40, 00 00 40 40]
  
  Split into streams:
    Stream 0: [00, 00, 00]        (MSBs)
    Stream 1: [00, 00, 40]
    Stream 2: [80, 00, 40]
    Stream 3: [3f, 40, 40]        (LSBs)
    
  Output: [00 00 00 00 00 40 80 00 40 3f 40 40]
```

**Algorithm:**
```rust
fn encode_byte_stream_split<T: AsBytes>(values: &[T]) -> Vec<u8> {
    let num_bytes = size_of::<T>();
    let mut streams: Vec<Vec<u8>> = vec![Vec::new(); num_bytes];
    
    for value in values {
        let bytes = value.as_bytes();
        for (i, byte) in bytes.iter().enumerate() {
            streams[i].push(*byte);
        }
    }
    
    streams.into_iter().flatten().collect()
}
```

---

## 7. DICTIONARY_RLE Encoding

Dictionary with run-length encoding. Best for low-cardinality data.

**When:**
- Cardinality < 256 values
- String or small data types
- Repetitive patterns

**Format:**
```
[dict_size: U32LE]

For each dictionary entry:
  [value_length: U16LE]
  [value: bytes]

[index_data: encoded as RLE]

Example (String):
  Dictionary: ["red", "green", "blue"]
  Values: ["red", "red", "green", "blue", "blue", "blue"]
  Indices: [0, 0, 1, 2, 2, 2]
  
  Encoded as RLE:
    run_len=2, index=0
    run_len=1, index=1
    run_len=3, index=2
```

---

## 8. PASSTHROUGH Encoding

No encoding applied. Pre-compressed or incompressible data.

**When:**
- Already compressed (JPEG, MP4, etc.)
- High-entropy blobs
- Binary attachments

**Format:**
```
Raw bytes, no transformation.
```

---

## Varint Encoding

Variable-length integer encoding for efficient storage.

```rust
fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    
    while value > 0x7F {
        result.push((value as u8) | 0x80);
        value >>= 7;
    }
    result.push(value as u8);
    
    result
}

fn decode_varint(data: &[u8]) -> u64 {
    let mut result = 0u64;
    let mut shift = 0;
    
    for &byte in data {
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    
    result
}
```

---

## SIMD Optimization Opportunities

1. **BIT_PACKED**: SIMD loop unroll and bit extraction
2. **RLE**: SIMD comparison for run detection
3. **DELTA_BINARY**: SIMD addition chains
4. **BYTE_STREAM_SPLIT**: SIMD shuffling

---

**End of Encoding Specification**
