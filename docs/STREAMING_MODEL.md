# Streaming Model

## Design Goal

QRD is built for workloads where data arrives incrementally and must be persisted without buffering the full dataset in memory.

## Row Group Semantics

QRD writes data into row groups with a fixed or adaptive row count. Each row group is a self-contained unit that can be written, compressed, and optionally recovered independently.

- Row groups bound memory usage on write
- Row group headers and metadata are included in the file stream
- Partial rows are not allowed across row group boundaries

## Write Model

The write model is:

1. Receive a row
2. Buffer it in the current row group
3. When the group reaches the target size:
   - transpose rows into columns
   - encode each column
   - compress the encoded payload
   - write the row group to the file stream

This model supports streaming ingestion with predictable resource use.

## Read Model

Readers obtain file structure from the footer and operate in one of these modes:

- full scan
- column projection
- row group projection
- footer-only metadata inspection

Because row groups include per-column offsets, readers can seek directly to relevant chunks.

## Bounded-Memory Behavior

Memory usage is bounded by:

- current row group buffer size on writes
- active column chunks on reads
- selected columns for partial reads

Large datasets are handled by streaming row groups sequentially, not by loading the whole file.

## Partial Read Guarantees

A partial read uses metadata to identify the offsets of requested columns, minimizing disk or network I/O. This is critical for edge and telemetry workloads where only a subset of fields is required.

## Summary

The QRD streaming model is intentionally narrow:

- writer memory ≈ row_group_size × average_row_size
- reader memory ≈ active columns × active row groups
- format supports append-efficient streaming and bounded-memory access
