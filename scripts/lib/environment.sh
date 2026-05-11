#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$ROOT_DIR"

write_environment_report() {
  local report_dir="$1"
  local rust_version
  local cargo_version
  local git_commit
  local git_branch
  local os_name
  local os_version

  rust_version="$(rustc --version 2>/dev/null || true)"
  cargo_version="$(cargo --version 2>/dev/null || true)"
  git_commit="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
  git_branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)"
  os_name="$(uname -s)"
  os_version="$(uname -r)"

  cat > "$report_dir/environment.txt" <<EOF
rust_version: $rust_version
cargo_version: $cargo_version
git_commit: $git_commit
git_branch: $git_branch
os_name: $os_name
os_version: $os_version
build_time: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
EOF
}
