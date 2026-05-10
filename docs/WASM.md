# WASM

## Overview

QRD supports a WASM execution path for browser and Node.js runtimes. The WASM path reuses the Rust core engine and provides a portable binary format for client-side applications.

## Build Path

The WASM target is produced from the Rust core and packaged in `sdk/typescript/`.

- compile the core to WASM
- expose a minimal runtime API
- keep binding logic thin

## Browser Use Cases

WASM is appropriate for:

- client-side telemetry collection
- browser feature inspection
- local analytics and preview
- offline-capable dashboards

## Runtime Constraints

WASM runtimes have different memory and system interfaces than native targets.

- use small row group sizes for low-memory browsers
- avoid large in-memory datasets in a single tab
- prefer partial reads and schema-only inspection

## Packaging Notes

The TypeScript/WASM layer should provide:

- package metadata for npm
- browser-compatible bindings
- Node.js compatibility
- documented runtime requirements

## Stability Note

The WASM path is part of the SDK portfolio, but actual runtime stability depends on ongoing build and packaging work.
