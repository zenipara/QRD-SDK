#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib/logging.sh"
source "$SCRIPT_DIR/lib/tools.sh"
source "$SCRIPT_DIR/lib/helpers.sh"

TOOLS=(
  cargo-audit
  cargo-deny
  cargo-nextest
  cargo-fuzz
  cargo-llvm-cov
  cargo-tarpaulin
  cargo-miri
  cargo-outdated
  cargo-udeps
)

log_step "Installing or validating QRD-SDK tooling"

for tool in "${TOOLS[@]}"; do
  if command -v "$tool" >/dev/null 2>&1; then
    log_info "Found $tool"
    continue
  fi

  log_warn "$tool is missing. Attempting install."
  if cargo install --locked "$tool"; then
    log_pass "$tool installed"
  else
    log_fail "Failed to install $tool"
  fi
done

log_pass "Toolchain validation complete"
