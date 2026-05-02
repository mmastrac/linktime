#!/usr/bin/env bash
set -xeuo pipefail

RUSTFLAGS="-Z sanitizer=address" cargo +nightly run -p ctor --example ctor-example --target $TARGET
RUSTFLAGS="-Z sanitizer=address" cargo +nightly run -p ctor --example ctor-advanced --target $TARGET
RUSTFLAGS="-Z sanitizer=address" cargo +nightly run -p link-section --example link-section-example --target $TARGET
