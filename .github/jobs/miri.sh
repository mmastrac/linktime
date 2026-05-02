#!/usr/bin/env bash
set -xeuo pipefail

# https://blog.rust-lang.org/2022/09/15/const-eval-safety-rule-revision/
export RUSTFLAGS="-Z extra-const-ub-checks"
# https://doc.rust-lang.org/nightly/std/ptr/index.html#strict-provenance
export MIRIFLAGS="-Zmiri-permissive-provenance"

cargo clean

cargo miri test

miri_crates=(
  tests/ctor/edition-2018
  tests/ctor/priority
  tests/link_section/basic
)
for dir in "${miri_crates[@]}"; do
  (cd "$dir" && cargo miri run --target "$TARGET")
done
