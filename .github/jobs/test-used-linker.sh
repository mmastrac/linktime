#!/usr/bin/env bash
set -xeuo pipefail

RUSTDOCFLAGS='--cfg linktime_used_linker' RUSTFLAGS='--cfg linktime_used_linker' cargo test --target $TARGET
