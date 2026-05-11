#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_outdated() {
  log_step "Running cargo outdated"

  if ! check_tool "cargo-outdated"; then
    log_fail "cargo-outdated not found"
    return 1
  fi

  local start_time=$(get_timestamp)
  if cargo outdated --workspace --exit-code 1 2>&1; then
    local duration=$(calculate_duration "$start_time")
    log_pass "cargo outdated completed in ${duration}s"
    return 0
  else
    local duration=$(calculate_duration "$start_time")
    log_warn "cargo outdated found updates after ${duration}s"
    # Don't fail on outdated dependencies, just warn
    return 0
  fi
}

