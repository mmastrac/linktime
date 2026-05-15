#!/usr/bin/env bash
set -xeuo pipefail

RUSTDOCFLAGS='--cfg linktime_used_linker' \
    RUSTFLAGS='-D warnings --cfg linktime_used_linker' \
    cargo test --no-fail-fast --target "$TARGET"
