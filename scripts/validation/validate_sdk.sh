#!/usr/bin/env bash
set -euo pipefail

echo "Validating SDKs..."

# Validate Python SDK
cd sdk/python
python -m pytest

# Validate Go SDK
cd ../go
go test ./...

# Validate TypeScript SDK
cd ../typescript
npm test

echo "SDK validation completed."