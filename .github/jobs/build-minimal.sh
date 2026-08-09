#!/usr/bin/env bash
set -xeuo pipefail

# Remove Cargo.lock files for testing down-level Rust versions
rm Cargo.lock || true
find tests -name Cargo.lock -not -path '*/target/*' -delete

# Standalone crok, the in-tree harness needs a newer rustc
find tests -name '*.crok' -not -path '*/target/*' -not -path '*/.*' -print0 | sort -z |
  xargs -0 crok --timeout 300
