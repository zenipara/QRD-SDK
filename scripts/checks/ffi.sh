#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_ffi() {
  log_step "Running FFI validation"

  if ! command -v cargo &> /dev/null; then
    log_fail "cargo not found"
    return 1
  fi

  local start_time=$(get_timestamp)

  # Build FFI library
  cd "$REPO_ROOT/core/qrd-ffi"
  if cargo build --release 2>&1; then
    local duration=$(calculate_duration "$start_time")
    log_pass "FFI build completed in ${duration}s"

    # Verify library output
    local lib_name
    case "$OSTYPE" in
      linux-gnu*) lib_name="libqrd_ffi.so" ;;
      darwin*) lib_name="libqrd_ffi.dylib" ;;
      *) lib_name="libqrd_ffi.so" ;;
    esac

    if [ -f "target/release/$lib_name" ]; then
      log_info "FFI library verified: $lib_name"
      return 0
    else
      log_fail "FFI library not found: $lib_name"
      return 1
    fi
  else
    local duration=$(calculate_duration "$start_time")
    log_fail "FFI build failed after ${duration}s"
    return 1
  fi
}

