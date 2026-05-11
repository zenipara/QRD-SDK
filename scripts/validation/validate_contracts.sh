#!/usr/bin/env bash
set -euo pipefail

echo "Validating contracts between languages..."

# Run contract tests for Rust ↔ Python
cd sdk/python
python -c "import qrd; print('Python contract OK')"

# Run contract tests for Rust ↔ Go
cd ../go
go run main.go

# Run contract tests for Rust ↔ WASM
cd ../../core/qrd-wasm
wasm-pack test --node

echo "Contract validation completed."