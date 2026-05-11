#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_release() {
  log_step "Running release profile validation"

  if ! command -v cargo &> /dev/null; then
    log_fail "cargo not found"
    return 1
  fi

  local start_time=$(get_timestamp)

  # Build in release mode
  if cargo build --workspace --release --all-features 2>&1; then
    local duration=$(calculate_duration "$start_time")
    log_pass "release build completed in ${duration}s"

    # Verify release artifacts exist
    if [ -f "target/release/qrd-core" ] || [ -f "target/release/libqrd_core.rlib" ]; then
      log_info "release artifacts verified"
      return 0
    else
      log_fail "release artifacts not found"
      return 1
    fi
  else
    local duration=$(calculate_duration "$start_time")
    log_fail "release build failed after ${duration}s"
    return 1
  fi
}

