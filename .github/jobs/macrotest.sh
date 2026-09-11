#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

# `macrotest::expand` runs cargo-expand
cargo test --no-fail-fast --target "$TARGET" -- --ignored
