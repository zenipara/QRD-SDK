#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_fuzz() {
  log_step "Running fuzz testing"

  if ! check_tool "cargo-fuzz"; then
    log_fail "cargo-fuzz not found"
    return 1
  fi

  local start_time=$(get_timestamp)
  local fuzz_dir="$REPO_ROOT/core/qrd-core/fuzz"

  # Check if fuzz targets exist
  if [ ! -d "$fuzz_dir" ]; then
    log_warn "no fuzz targets found, skipping fuzz testing"
    return 0
  fi

  # Run a quick fuzz test on available targets
  local targets
  mapfile -t targets < <(cargo fuzz list 2>/dev/null || echo "")

  if [ ${#targets[@]} -eq 0 ]; then
    log_warn "no fuzz targets available"
    return 0
  fi

  local failed=0
  for target in "${targets[@]}"; do
    log_info "Testing fuzz target: $target"
    if timeout 30 cargo fuzz run "$target" -- -runs=100 2>&1; then
      log_pass "fuzz target '$target' passed"
    else
      log_fail "fuzz target '$target' failed"
      ((failed++))
    fi
  done

  local duration=$(calculate_duration "$start_time")

  if [ $failed -eq 0 ]; then
    log_pass "fuzz testing completed in ${duration}s"
    return 0
  else
    log_fail "fuzz testing failed: $failed targets failed"
    return 1
  fi
}

