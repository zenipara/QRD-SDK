# Compression

## Design Overview

QRD separates encoding from compression. Data is first transformed into a representation optimized for compressibility, then a codec is selected and applied at the column chunk level.

## Codec Selection

QRD supports:

- `NONE`
- `ZSTD`
- `LZ4`
- `GZIP` (reserved)

The runtime chooses a codec based on:

- data entropy
- latency requirements
- payload size
- workload profile

### ZSTD

- best for archive-quality storage and higher compression ratios
- offers a balanced speed/compression tradeoff

### LZ4

- best for low-latency write/read paths
- optimized for streaming and partial read workloads

## Chunk-Level Compression

Each column chunk is compressed independently. This enables:

- partial reads without decompressing unrelated data
- parallel decompression
- independent codec choices per column

## Compression Pipeline

```
column values → encoding → compression → encrypted payload
```

Compression is applied after the encoding stage and before optional encryption.

## Compression Metadata

Each column chunk stores metadata about:

- the encoding type
- the compression codec
- uncompressed length
- compressed length
- statistics and checksum

This metadata is required for safe decoding and skipping of chunks.

## Performance Considerations

Compression improves storage efficiency but adds CPU cost. QRD is designed to make the tradeoff explicit:

- use LZ4 for write-heavy, latency-sensitive streams
- use ZSTD for archive or analytics workloads

## Reproducibility

Compression metadata and codec selection are deterministic at the engine level so that identical input produces the same output across bindings.
