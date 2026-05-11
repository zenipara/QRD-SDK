#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_audit() {
  log_step "Running cargo audit"

  if ! check_tool "cargo-audit"; then
    log_fail "cargo-audit not found"
    return 1
  fi

  local start_time=$(get_timestamp)
  if cargo audit 2>&1; then
    local duration=$(calculate_duration "$start_time")
    log_pass "cargo audit completed in ${duration}s"
    return 0
  else
    local duration=$(calculate_duration "$start_time")
    log_fail "cargo audit failed after ${duration}s"
    return 1
  fi
}

