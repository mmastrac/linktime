#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

cargo build

ctor_examples=(ctor-example ctor-advanced)
for example in "${ctor_examples[@]}"; do
  cargo run -p ctor --example "$example" --target "$TARGET"
done
