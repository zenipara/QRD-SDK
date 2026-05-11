#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_coverage() {
  log_step "Running coverage analysis"

  if ! check_tool "cargo-tarpaulin"; then
    log_fail "cargo-tarpaulin not found"
    return 1
  fi

  local start_time=$(get_timestamp)
  local coverage_dir="$CHECK_DIR/../artifacts/coverage"
  mkdir -p "$coverage_dir"

  if cargo tarpaulin --workspace --all-features --out Html --output-dir "$coverage_dir" --exclude-files "*/tests/*" --exclude-files "*/examples/*" --exclude-files "*/benches/*" --timeout 300 2>&1; then
    local duration=$(calculate_duration "$start_time")
    log_pass "coverage analysis completed in ${duration}s"

    # Check coverage thresholds
    local coverage_file="$coverage_dir/tarpaulin-report.html"
    if [ -f "$coverage_file" ]; then
      # Extract coverage percentage (simplified)
      local coverage_pct=$(grep -oP 'coverage: \K[0-9.]+' "$coverage_file" | head -1 || echo "0")
      log_info "Coverage: ${coverage_pct}%"

      # Check minimum thresholds
      if (( $(echo "$coverage_pct < 95" | bc -l 2>/dev/null || echo "1") )); then
        log_warn "Coverage below 95% threshold: ${coverage_pct}%"
      fi
    fi

    return 0
  else
    local duration=$(calculate_duration "$start_time")
    log_fail "coverage analysis failed after ${duration}s"
    return 1
  fi
}

