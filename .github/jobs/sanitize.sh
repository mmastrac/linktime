#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

# pkg:example pairs for address-sanitizer smoke runs
sanitize_runs=(
  "ctor:ctor-example"
  "ctor:ctor-advanced"
  "link-section:link-section-example"
  "link-section:link-section-ref"
  "link-section:link-section-mut"
  "link-section:link-section-movable"
  "link-section:link-section-movable-no-macro"
)

for spec in "${sanitize_runs[@]}"; do
  pkg="${spec%%:*}"
  example="${spec#*:}"
  RUSTFLAGS="--cfg linktime_asan" \
    cargo +nightly rustc -p "$pkg" --example "$example" --target "$TARGET" -- \
    -Z sanitizer=address -Z "crate-attr=feature(sanitize)"
  "target/${TARGET}/debug/examples/${example}"
done
