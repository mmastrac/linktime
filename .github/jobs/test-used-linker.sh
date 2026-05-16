#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

RUSTDOCFLAGS='--cfg linktime_used_linker' \
    RUSTFLAGS='-D warnings --cfg linktime_used_linker' \
    cargo test --no-fail-fast --target "$TARGET"
