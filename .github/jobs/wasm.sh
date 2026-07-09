#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# crok-lib needs a newer rustc than the 1.85 MSRV leg, so the runnable suite is
# stable-only. `WASM_PLATFORM=1` marks this as the dedicated wasm leg.
if [ "${TOOLCHAIN:-}" = "stable" ]; then
  WASM_PLATFORM=1 cargo test -p tests -- wasm::
else
  cd "$ROOT/tests/wasm/rust"
  cargo build --target wasm32-unknown-unknown
  cargo build --target wasm32-wasip1
  cargo build --target wasm32-wasip2
fi

echo "Done."
