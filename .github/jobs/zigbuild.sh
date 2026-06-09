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
  scattered-collect-intern-strings
  scattered-collect-slice
  scattered-collect-sorted-slice
  scattered-collect-referenced-slice
  scattered-collect-sorted-referenced-slice
  scattered-collect-map
  scattered-collect-set
  scattered-collect-iterable
)

for bin in "${zig_examples[@]}"; do
  sleep .1
  echo "Running example: ${bin}"
  sleep .1
  "target/${TARGET}/debug/examples/${bin}"
done
