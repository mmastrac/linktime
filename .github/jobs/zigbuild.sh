#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

cargo zigbuild --workspace --bins --examples --target "$TARGET"

zig_examples=(
  ctor-basic
  ctor-example
  ctor-advanced
  link-section-const
  link-section-dyn
  link-section-empty
  link-section-example
  link-section-mut
)

for bin in "${zig_examples[@]}"; do
  sleep .1
  echo "Running example: ${bin}"
  sleep .1
  "target/${TARGET}/debug/examples/${bin}"
done
