#!/usr/bin/env bash
set -xeuo pipefail

RUSTFLAGS='-D warnings' cargo test --no-fail-fast --target "$TARGET"
