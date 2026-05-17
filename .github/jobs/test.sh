#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

RUSTFLAGS='-D warnings' cargo test --no-fail-fast --target "$TARGET"
