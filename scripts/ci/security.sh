#!/usr/bin/env bash
set -euo pipefail

echo "Running security audits..."

cargo audit
cargo deny check
cargo vet

echo "Security audits passed."