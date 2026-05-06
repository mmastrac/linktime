#!/usr/bin/env bash
set -xeuo pipefail

cargo zigbuild --workspace --bins --examples --target "$TARGET"

zig_examples=(
  ctor-basic
  ctor-example
  ctor-advanced
  ctor-dynamic
  ctor-statics
  link-section-example
)

for bin in "${zig_examples[@]}"; do
  sleep .1
  echo "Running example: ${bin}"
  sleep .1
  "target/${TARGET}/debug/examples/${bin}"
done
