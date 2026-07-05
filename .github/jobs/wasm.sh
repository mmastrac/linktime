#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if [ "$TOOLCHAIN" = "stable" ]; then
  cargo test -p tests -- wasm
fi

cd "$ROOT/tests/wasm/rust"
cargo build --target wasm32-unknown-unknown
cargo build --target wasm32-wasip1
cargo build --target wasm32-wasip2

# WASI smoketest
wasmtime run target/wasm32-wasip1/debug/wasm_rust.wasm
wasmtime run target/wasm32-wasip2/debug/wasm_rust.wasm

# WASI via node
npx tsx "$ROOT/tests/wasm/js/test-wasm32-unknown-unknown.mts"
npx tsx "$ROOT/tests/wasm/js/test-wasm32-wasi.mts"
cd "$ROOT"

echo "Done."
