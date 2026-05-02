#!/usr/bin/env bash
set -xeuo pipefail

if [ "$TOOLCHAIN" != "stable" ]; then
  cargo clean
fi
cargo clippy --examples --bins --all --target "$TARGET" -- ${CLIPPY_LINTS}
cargo fmt --check --all

# Ensure generated docs are up to date.
cargo generate-docs
if ! git diff --exit-code; then
  echo
  echo "ERROR: Generated docs are out of date."
  echo "Run: cargo generate-docs"
  echo
  git diff
  exit 1
fi
