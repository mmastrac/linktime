#!/usr/bin/env bash
set -xeuo pipefail

# Remove Cargo.lock for testing down-level Rust versions
rm Cargo.lock
# May need to rebuild when beta/nightly changes
cargo clean
cargo build

ctor_examples=(ctor-example ctor-advanced)
for example in "${ctor_examples[@]}"; do
  cargo run -p ctor --example "$example" --target "$TARGET"
done
