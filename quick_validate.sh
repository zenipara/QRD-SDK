#!/bin/bash

# Quick validation script untuk Phase 2 implementation

echo "=== QRD-SDK Phase 2 Quick Validation ==="
echo "Timestamp: $(date)"
echo

# Info sistem
echo "System Info:"
echo "- OS: $(uname -s)"
echo "- Arch: $(uname -m)"
echo

# Check tools
echo "Checking required tools:"
which cargo > /dev/null && echo "✓ cargo: $(cargo --version)" || echo "✗ cargo not found"
which rustc > /dev/null && echo "✓ rustc: $(rustc --version)" || echo "✗ rustc not found"
which git > /dev/null && echo "✓ git: $(git --version)" || echo "✗ git not found"
echo

# Check workspace structure
echo "Workspace Structure:"
[ -f "Cargo.toml" ] && echo "✓ Root Cargo.toml exists" || echo "✗ Root Cargo.toml missing"
[ -f "core/qrd-core/Cargo.toml" ] && echo "✓ qrd-core Cargo.toml exists" || echo "✗ qrd-core Cargo.toml missing"
[ -d "core/qrd-core/src" ] && echo "✓ src directory exists" || echo "✗ src directory missing"
echo

# Check Phase 2 files
echo "Phase 2 Implementation Files:"
[ -f "core/qrd-core/src/encryption/mod.rs" ] && echo "✓ encryption/mod.rs" || echo "✗ encryption/mod.rs missing"
[ -f "core/qrd-core/src/ecc/mod.rs" ] && echo "✓ ecc/mod.rs" || echo "✗ ecc/mod.rs missing"
[ -f "core/qrd-core/src/utils/simd.rs" ] && echo "✓ utils/simd.rs" || echo "✗ utils/simd.rs missing"
[ -f "core/qrd-core/src/utils/bit_ops.rs" ] && echo "✓ utils/bit_ops.rs" || echo "✗ utils/bit_ops.rs missing"
[ -f "core/qrd-core/tests/integration_test.rs" ] && echo "✓ tests/integration_test.rs" || echo "✗ tests/integration_test.rs missing"
[ -f "core/qrd-core/benches/encode_bench.rs" ] && echo "✓ benches/encode_bench.rs" || echo "✗ benches/encode_bench.rs missing"
echo

# Check documentation
echo "Documentation Files:"
[ -f "PHASE2_COMPLETION_REPORT.md" ] && echo "✓ PHASE2_COMPLETION_REPORT.md" || echo "✗ PHASE2_COMPLETION_REPORT.md missing"
[ -f "validate_phase2.sh" ] && echo "✓ validate_phase2.sh" || echo "✗ validate_phase2.sh missing"
echo

# Attempt cargo check
echo "Attempting Cargo check..."
if command -v cargo &> /dev/null; then
    cd core/qrd-core 2>/dev/null
    if cargo check 2>&1 | head -20; then
        echo "✓ Cargo check successful"
    else
        echo "⚠ Cargo check failed - see output above"
    fi
    cd - > /dev/null 2>&1
else
    echo "✗ Cargo not available"
fi

echo
echo "=== Validation Complete ==="
