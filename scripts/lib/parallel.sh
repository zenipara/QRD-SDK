#!/usr/bin/env bash
set -euo pipefail

run_in_parallel() {
  local fn="$1"
  shift
  if command -v parallel >/dev/null 2>&1; then
    printf '%s\n' "$@" | parallel "$fn {}"
  else
    for item in "$@"; do
      "$fn" "$item"
    done
  fi
}
