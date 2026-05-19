#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

export RUSTDOCFLAGS="--cfg linktime_used_linker \
    -Z crate-attr=feature(used_with_arg) \
    -Z crate-attr=allow(unused_features) \
    -Z crate-attr=allow(duplicate_features)"
export RUSTFLAGS="-D warnings --cfg linktime_used_linker \
    -Z crate-attr=feature(used_with_arg) \
    -Z crate-attr=allow(unused_features) \
    -Z crate-attr=allow(duplicate_features)"

cargo test --no-fail-fast --target "$TARGET"
