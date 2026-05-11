#!/usr/bin/env bash
set -euo pipefail

assert_tool() {
  local tool="$1"
  if ! command -v "$tool" >/dev/null 2>&1; then
    log_fail "Required tool not found: $tool"
    exit 1
  fi
}

ensure_tool() {
  local tool="$1"
  if ! command -v "$tool" >/dev/null 2>&1; then
    log_warn "$tool is missing"
    return 1
  fi
  return 0
}

check_tool() {
  local tool="$1"
  if command -v "$tool" >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

is_nightly_available() {
  rustup toolchain list | grep -q '^nightly' || return 1
  return 0
}
