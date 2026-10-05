#!/usr/bin/env bash
# Runs a CI task sequentially across a focused feature matrix:
# all features, each feature individually, and no features.
# Features are discovered from Cargo.toml, so new features need no CI edits.
# Usage: scripts/ci.sh <check|clippy|test|examples|msrv>
set -euo pipefail
cd "$(dirname "$0")/.."

TOOLCHAIN="${TOOLCHAIN:-stable}"
MSRV="${MSRV:-1.74.0}"

features() {
  awk '/^\[features\]/{f=1;next} /^\[/{f=0} f && /^[A-Za-z0-9_-]+[ ]*=/{sub(/[ ]*=.*/,"");if($0!="default")print}' Cargo.toml
}

# Prints one flag set per line: all, none, then each feature alone.
configs() {
  echo "--all-features"
  echo "--no-default-features"
  local f
  for f in $(features); do echo "--no-default-features --features $f"; done
}

run() {
  echo "+ $*" >&2
  "$@"
}

each_config() {
  local flags
  while IFS= read -r flags; do
    # shellcheck disable=SC2086
    "$@" $flags
  done < <(configs)
}

check()  { run cargo "+$TOOLCHAIN" check --all-targets --locked "$@"; }
clippy() { run cargo "+$TOOLCHAIN" clippy --all-targets --locked "$@" -- -D warnings; }
tests()  { run cargo "+$TOOLCHAIN" test --locked "$@"; }
msrv()   { run cargo "+$MSRV" check --lib --locked "$@"; }

each_example() {
  local manifest
  for manifest in examples/*/Cargo.toml; do
    "$@" "$manifest"
  done
}

example_test() { run cargo "+$TOOLCHAIN" test --locked --manifest-path "$1"; }
example_msrv() { run cargo "+$MSRV" check --locked --manifest-path "$1"; }

case "${1:-}" in
  check)    each_config check ;;
  clippy)   each_config clippy ;;
  test)     each_config tests ;;
  examples) each_example example_test ;;
  msrv)     each_config msrv; each_example example_msrv ;;
  *) echo "usage: $0 <check|clippy|test|examples|msrv>" >&2; exit 2 ;;
esac
