#!/usr/bin/env bash
set -xeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$ROOT/tests/wasm/rust"
cargo build --target wasm32-unknown-unknown
cargo build --target wasm32-wasip1
cargo build --target wasm32-wasip2

# WASI smoketest
wasmtime run target/wasm32-wasip1/debug/wasm_rust.wasm

# WASI via node
npx tsx "$ROOT/tests/wasm/js/test-wasm32-unknown-unknown.mts"
npx tsx "$ROOT/tests/wasm/js/test-wasm32-wasi.mts"
cd "$ROOT"

# We don't have a way to initialize the runtime yet...
cd "$ROOT/tests/link_section/wasm"
cargo build --target wasm32-unknown-unknown
# wasmtime run target/wasm32-unknown-unknown/debug/tests_link_section_wasm.wasm
cargo build --target wasm32-wasip1
# wasmtime run target/wasm32-wasip1/debug/tests_link_section_wasm.wasm || echo "WASI failed"
cd "$ROOT"

echo "Done."
