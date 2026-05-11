#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_udeps() {
  log_step "Running cargo udeps"

  if ! check_tool "cargo-udeps"; then
    log_fail "cargo-udeps not found"
    return 1
  fi

  local start_time=$(get_timestamp)
  if cargo +nightly udeps --workspace --all-targets --all-features 2>&1; then
    local duration=$(calculate_duration "$start_time")
    log_pass "cargo udeps completed in ${duration}s"
    return 0
  else
    local duration=$(calculate_duration "$start_time")
    log_fail "cargo udeps failed after ${duration}s"
    return 1
  fi
}

