#!/usr/bin/env bash
set -xeuo pipefail

# https://blog.rust-lang.org/2022/09/15/const-eval-safety-rule-revision/
export RUSTFLAGS="-Z extra-const-ub-checks"
# https://doc.rust-lang.org/nightly/std/ptr/index.html#strict-provenance
export MIRIFLAGS="-Zmiri-permissive-provenance"

# May need to rebuild when beta/nightly changes
cargo clean

cargo miri test

cd tests/ctor/edition-2018
cargo miri run --target $TARGET
cd ../../..

cd tests/ctor/priority
cargo miri run --target $TARGET
cd ../../..

cd tests/link_section/basic
cargo miri run --target $TARGET
cd ../../..
