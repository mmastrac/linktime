#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

# `macrotest::expand` runs cargo-expand
# --tests skips doctests
cargo test --no-fail-fast --tests --workspace --exclude tests --target "$TARGET" -- --ignored
