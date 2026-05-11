#!/bin/bash
#
# Coverage Measurement Script for QRD SDK
# 
# Measures code coverage and enforces thresholds:
# - Line coverage >= 80%
# - Branch coverage >= 70%
#
# Usage:
#   ./measure_coverage.sh              # Measure coverage
#   ./measure_coverage.sh --enforce    # Enforce thresholds (fail if below)
#   ./measure_coverage.sh --html       # Generate HTML report

set -e

COVERAGE_DIR="target/coverage"
ENFORCE_THRESHOLD=${1:-}
MIN_LINE_COVERAGE=80
MIN_BRANCH_COVERAGE=70

echo "================================"
echo "QRD SDK Code Coverage Measurement"
echo "================================"
echo ""

# Ensure cargo and tarpaulin are available
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "ERROR: cargo-tarpaulin not found. Install with: cargo install cargo-tarpaulin"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "ERROR: cargo not found. Please install Rust toolchain."
    exit 1
fi

# Create coverage directory
mkdir -p "$COVERAGE_DIR"

echo "Running tests with coverage instrumentation..."
echo ""

# Run coverage measurement with tarpaulin
# Options:
#   -p qrd-core: measure only qrd-core package
#   --out Xml: output as XML for processing
#   --timeout 300: 5 minute timeout per test
#   --run-types Tests: measure test coverage
cargo tarpaulin \
    -p qrd-core \
    --out Xml \
    --output-dir "$COVERAGE_DIR" \
    --timeout 300 \
    --exclude-files tests/ \
    --run-types Tests \
    2>&1 | tee "$COVERAGE_DIR/tarpaulin.log"

echo ""
echo "Coverage measurement completed."
echo ""

# Parse coverage from XML output
COVERAGE_XML="$COVERAGE_DIR/cobertura.xml"

if [ -f "$COVERAGE_XML" ]; then
    # Extract line-rate and branch-rate from cobertura XML
    echo "Parsing coverage metrics..."
    
    # Try to extract line coverage (this is a simplified extraction)
    LINE_COVERAGE=$(grep -oP 'line-rate="\K[^"]+' "$COVERAGE_XML" | head -1 || echo "unknown")
    BRANCH_COVERAGE=$(grep -oP 'branch-rate="\K[^"]+' "$COVERAGE_XML" | head -1 || echo "unknown")
    
    # Convert to percentage if we got decimal values
    if [[ "$LINE_COVERAGE" != "unknown" ]]; then
        LINE_COVERAGE_PCT=$(echo "$LINE_COVERAGE * 100" | bc | cut -d. -f1)
    else
        LINE_COVERAGE_PCT=0
    fi
    
    if [[ "$BRANCH_COVERAGE" != "unknown" ]]; then
        BRANCH_COVERAGE_PCT=$(echo "$BRANCH_COVERAGE * 100" | bc | cut -d. -f1)
    else
        BRANCH_COVERAGE_PCT=0
    fi
    
    echo "Coverage Report:"
    echo "  Line Coverage:     $LINE_COVERAGE_PCT% (target: ${MIN_LINE_COVERAGE}%)"
    echo "  Branch Coverage:   $BRANCH_COVERAGE_PCT% (target: ${MIN_BRANCH_COVERAGE}%)"
    echo ""
    
    # Check thresholds if --enforce flag is set
    if [[ "$ENFORCE_THRESHOLD" == "--enforce" ]]; then
        FAILED=0
        
        if (( LINE_COVERAGE_PCT < MIN_LINE_COVERAGE )); then
            echo "❌ FAILED: Line coverage ${LINE_COVERAGE_PCT}% is below threshold ${MIN_LINE_COVERAGE}%"
            FAILED=1
        else
            echo "✅ PASSED: Line coverage ${LINE_COVERAGE_PCT}% meets threshold ${MIN_LINE_COVERAGE}%"
        fi
        
        if (( BRANCH_COVERAGE_PCT < MIN_BRANCH_COVERAGE )); then
            echo "❌ FAILED: Branch coverage ${BRANCH_COVERAGE_PCT}% is below threshold ${MIN_BRANCH_COVERAGE}%"
            FAILED=1
        else
            echo "✅ PASSED: Branch coverage ${BRANCH_COVERAGE_PCT}% meets threshold ${MIN_BRANCH_COVERAGE}%"
        fi
        
        echo ""
        
        if [[ $FAILED -eq 1 ]]; then
            echo "Coverage thresholds not met. Failing CI."
            exit 1
        fi
    fi
    
else
    echo "Warning: Coverage XML not found at $COVERAGE_XML"
fi

echo "Coverage report saved to: $COVERAGE_DIR/"
echo ""
echo "To view detailed coverage:"
echo "  cat $COVERAGE_DIR/tarpaulin.log"
echo ""
