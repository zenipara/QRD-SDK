# Memory Model

## Overview

QRD is designed for low-memory deployment environments by making memory usage a function of the row group and selected columns, not the entire file.

## Writer Memory Flow

On the write side, memory is used for:

- row buffer
- column transposition buffers
- encoding buffer
- compression buffer
- optional encryption buffer

Memory peaks at the row group boundary. Once a row group is flushed, only metadata and footer buffers remain.

## Reader Memory Flow

On the read side, memory is used for:

- footer metadata
- selected row group headers
- selected column chunk payloads
- decompressed and decoded values for rows or columns

Partial reads reduce memory by skipping unrelated columns and row groups.

## Zero-Copy Principles

QRD reduces copies by:

- reading footer metadata separately from data payloads
- reusing compressed buffers when possible
- decoding only requested columns when supported by the runtime

## Bounded Memory Guarantees

The format guarantees that:

- writer memory is proportional to one row group
- reader memory is proportional to columns and row groups being processed
- streaming operations do not require full-file materialization

## Implications for Edge Environments

This model makes QRD suitable for:

- IoT gateways
- browser edge browsers
- mobile devices
- in-process analytics on constrained hosts
