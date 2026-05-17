#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

# Remove Cargo.lock for testing down-level Rust versions
rm Cargo.lock || true

minimal_crates=(
  tests/ctor/edition-2018
  tests/ctor/edition-2021
  tests/dtor/link-section
)
for dir in "${minimal_crates[@]}"; do
  (cd "$dir" && (rm Cargo.lock || true) && cargo run --target "$TARGET")
done
