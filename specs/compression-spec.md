# Compression Specification

**Document Version:** 1.0.0  
**Status:** Draft

## Overview

Compression in QRD is applied after encoding to further reduce data size. This document specifies the compression strategy and codecs.

## Core Principles

1. **Compress after encoding** - Use encoded data as compression input
2. **Skip incompressible data** - Detect high-entropy data and skip
3. **Adaptive selection** - Choose codec based on context
4. **Streaming friendly** - Block-based compression for memory efficiency

## Supported Codecs

### ZSTD (Zstandard)

**Characteristics:**
- High compression ratio (8-20x for text)
- Variable compression level (1-22)
- Fast decompression (most important for reads)
- Modern, well-maintained

**Default Levels:**
- Level 3: Balance (default)
- Level 1: Fast
- Level 10-22: Maximum compression (archive mode)

**Best for:**
- Text, JSON, CSV data
- Archive/cold storage
- Off-line batch processing
- Final compression pass

**Format:**
```
[frame_magic: 4 bytes] = 0x28, 0xB5, 0x2F, 0xFD
[frame_header: variable]
[data_blocks: variable]
[checksum: optional]
```

### LZ4

**Characteristics:**
- Ultra-fast compression (500+ MB/s)
- Low compression ratio (2-5x)
- Very fast decompression
- Dictionary support

**Default Level:**
- Level 4: Fast

**Best for:**
- Real-time streaming
- Interactive systems
- Network transmission
- High-throughput scenarios

**Format:**
```
[magic: 4 bytes] = 0x04, 0x22, 0x4D, 0x18 (with frame wrapper)
[frame_descriptor: variable]
[data_blocks: variable]
```

### NONE

**Usage:**
- Data already compressed (JPEG, MP4, etc.)
- High-entropy data
- Incompressible patterns

## Compression Selection Algorithm

```rust
fn select_compression_codec(
    column_data: &[u8],
    encoding: EncodingType,
) -> CompressionCodec {
    let entropy = calculate_entropy(column_data);
    
    // Don't compress already-compressed data
    if entropy > 7.5 {
        return CompressionCodec::None;
    }
    
    // Don't compress blobs/images
    if encoding == EncodingType::Passthrough {
        return CompressionCodec::None;
    }
    
    // Stream mode: prioritize throughput
    if writer_is_streaming() {
        return CompressionCodec::Lz4;  // Ultra-fast
    }
    
    // Archive mode: maximize compression
    return CompressionCodec::Zstd;  // Best ratio
}
```

## Entropy Threshold

Entropy calculation:

```
entropy = sum(p_i * log2(p_i)) for each byte value
where p_i = frequency of byte value i

Ranges:
- 0: No entropy (all bytes identical)
- 8: Maximum entropy (uniform distribution)

Threshold: 7.5 bits/byte indicates data unlikely to compress
           (typical for already-compressed data)
```

## Block Structure

Both ZSTD and LZ4 use block-based compression:

```
Compressed Block:
  [header: variable]
  [compressed_data: variable]
  [checksum: optional]
```

Benefits:
- Streaming decompression without loading entire file
- Parallel decompression of multiple blocks
- Corruption detection per block
- Graceful degradation on errors

## Compression Levels

### ZSTD Levels

| Level | Speed | Ratio | Use Case |
|-------|-------|-------|----------|
| 1 | Fastest | 2-3x | Real-time with some compression |
| 3 | Fast | 3-5x | Default balance |
| 6 | Normal | 5-8x | Good balance |
| 10 | Slow | 8-15x | Archive |
| 19 | Very Slow | 10-20x | Deep archive |

### LZ4 Levels

| Level | Speed | Ratio | Use Case |
|-------|-------|-------|----------|
| 4 | Ultra-fast | 2-3x | Streaming (default) |
| N/A | N/A | N/A | LZ4 has limited levels |

## Configuration

### Writer Configuration

```rust
pub struct CompressionConfig {
    /// Codec to use
    pub codec: CompressionCodec,
    
    /// Compression level
    pub level: u8,
    
    /// Enable adaptive selection
    pub adaptive: bool,
    
    /// Block size in bytes
    pub block_size: usize,
}
```

### Default Settings

- **Codec**: ZSTD
- **Level**: 3 (default balance)
- **Adaptive**: true (select based on data)
- **Block Size**: 64KB

## Performance

### Throughput (typical x86_64)

| Operation | Codec | Throughput |
|-----------|-------|-----------|
| Compress | ZSTD (lvl 3) | 200-500 MB/s |
| Compress | LZ4 (lvl 4) | 500+ MB/s |
| Decompress | ZSTD | 1-3 GB/s |
| Decompress | LZ4 | 3-10 GB/s |

### Compression Ratios

| Data Type | ZSTD | LZ4 |
|-----------|------|-----|
| JSON | 8-15x | 3-5x |
| CSV | 10-20x | 2-8x |
| UTF-8 Text | 5-10x | 2-4x |
| Binary data | 1.2-3x | 1.1-2x |
| Already compressed | 1.0-1.1x | 1.0-1.1x |

## Determinism

Compression MUST be deterministic:

```rust
// Same input → same compressed output
let data = b"hello world";
let compressed1 = compress(data, ZSTD, level=3)?;
let compressed2 = compress(data, ZSTD, level=3)?;
assert_eq!(compressed1, compressed2);  // Always true
```

Requirements:
- Fixed compression level (no random parameters)
- No timestamps in output
- Same data → identical bytes
- Verified in test suite

## Error Handling

### On Compression Failure

1. Log error
2. Skip compression (store uncompressed)
3. Set compression codec = NONE in metadata
4. Data remains readable

### On Decompression Failure

1. Check CRC checksum
2. Try recovery if ECC available
3. Return error if unrecoverable
4. File marked as corrupted

## Future Codecs

Potential future additions:
- **Brotli**: High ratio, slow (archival)
- **Deflate**: Compatibility with other systems
- **Snappy**: Fast alternative to LZ4
- **Custom**: Domain-specific compression

---

**End of Compression Specification**
