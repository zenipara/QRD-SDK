#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_stress() {
  log_step "Running stress testing"

  if ! command -v cargo &> /dev/null; then
    log_fail "cargo not found"
    return 1
  fi

  local start_time=$(get_timestamp)

  # Run stress tests (marked with #[ignore] and --ignored)
  if cargo test --workspace -- --ignored stress 2>&1; then
    local duration=$(calculate_duration "$start_time")
    log_pass "stress testing completed in ${duration}s"
    return 0
  else
    local duration=$(calculate_duration "$start_time")
    log_fail "stress testing failed after ${duration}s"
    return 1
  fi
}

