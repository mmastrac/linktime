#!/usr/bin/env bash
set -xeuo pipefail

# Remove Cargo.lock files for testing down-level Rust versions
rm Cargo.lock || true
find tests -name Cargo.lock -not -path '*/target/*' -delete

# ctor/dtor only, the rest exceed ctor's MSRV. edition-2024 needs 1.85+.
# Standalone crok, the in-tree harness needs a newer rustc
find tests/ctor tests/dtor -name '*.crok' \
  -not -path '*/target/*' -not -path '*/.*' -not -path '*/edition-2024/*' -print0 | sort -z |
  xargs -0 crok --timeout 300
