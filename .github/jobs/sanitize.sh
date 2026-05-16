#!/usr/bin/env bash
set -xeuo pipefail

# pkg:example pairs for address-sanitizer smoke runs
sanitize_runs=(
  "ctor:ctor-example"
  "ctor:ctor-advanced"
  "link-section:link-section-example"
  "link-section:link-section-ref"
  "link-section:link-section-mut"
  "link-section:link-section-movable"
)

for spec in "${sanitize_runs[@]}"; do
  pkg="${spec%%:*}"
  example="${spec#*:}"
  RUSTFLAGS="-Z sanitizer=address" cargo +nightly run -p "$pkg" --example "$example" --target "$TARGET"
done
