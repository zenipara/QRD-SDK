#!/usr/bin/env bash
set -euo pipefail

timer_start() {
  date +%s%3N
}

timer_elapsed_ms() {
  local start="$1"
  local now
  now=$(date +%s%3N)
  echo $((now - start))
}

calculate_duration() {
  timer_elapsed_ms "$1"
}

format_duration() {
  local ms="$1"
  local seconds=$((ms / 1000))
  local remainder=$((ms % 1000))
  printf "%ds %dms" "$seconds" "$remainder"
}
