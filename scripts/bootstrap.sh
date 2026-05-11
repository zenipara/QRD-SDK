#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib/logging.sh"
source "$SCRIPT_DIR/lib/tools.sh"
source "$SCRIPT_DIR/lib/helpers.sh"

log_step "Bootstrapping QRD-SDK validation environment"

if [[ ! -d "$SCRIPT_DIR" ]]; then
  log_fail "Unable to resolve scripts directory"
  exit 1
fi

log_info "Creating report directories"
create_report_directories "$SCRIPT_DIR/reports/latest"

log_info "Checking required tools"
assert_tool "bash"
assert_tool "cargo"
assert_tool "rustc"
assert_tool "git"
assert_tool "python3"

log_info "Bootstrapping complete"
log_pass "QRD-SDK validation environment is ready"
