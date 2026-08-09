#!/usr/bin/env bash
set -xeuo pipefail

# Remove Cargo.lock files for testing down-level Rust versions
rm Cargo.lock || true
find tests -name Cargo.lock -not -path '*/target/*' -delete

# Only these run on ctor's MSRV, the rest resolve deps too new for 1.65
minimal_crates=(
  tests/ctor/edition-2018
  tests/ctor/edition-2021
  tests/dtor/link-section
)

# Standalone crok, the in-tree harness needs a newer rustc
find "${minimal_crates[@]}" -name '*.crok' \
  -not -path '*/target/*' -not -path '*/.*' -print0 | sort -z |
  xargs -0 crok --timeout 300
