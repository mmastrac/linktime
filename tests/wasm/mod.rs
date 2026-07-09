//! WASM runtime tests.
//!
//! These build the crates under `tests/wasm/` for the wasm targets and run them
//! under `wasmtime` and `node`, asserting on their output. The prerequisites
//! (`node`, `wasmtime`, and the `wasm32-*` targets) are probed up front.

use crok_lib::crok;

// Runs `wasm-rust` (ctor / dtor / link_section together) for all three wasm
// targets and checks the program output under both `wasmtime` and `node`.
//
// The two priority-1 ctors (`ctor_slices` and the `MOVABLE_LINK_SECTION` sort)
// have no ordering guarantee between them, so their lines are matched
// `unordered`; everything after is deterministic.
crok!(
    runtime,
    r#"
set RUSTFLAGS "";

# --- prerequisites: skip on non-wasm CI, require on the wasm CI leg ---
$ sh -c '[ -n "${CI:-}" ] && echo 1 || echo 0'
%SET is_ci
*
$ sh -c '[ -n "${WASM_PLATFORM:-}" ] && echo 1 || echo 0'
%SET require
*
if is_ci == "1" {
    if require == "0" {
        exit script;
    }
}
$ command -v node >/dev/null 2>&1 && command -v wasmtime >/dev/null 2>&1 && echo 1 || echo 0
%SET has_tools
*
$ n=$(rustup target list --installed 2>/dev/null | grep -cE '^wasm32-(unknown-unknown|wasip1|wasip2)$'); [ "$n" = 3 ] && echo 1 || echo 0
%SET has_targets
*
if has_tools == "0" {
    if require == "1" {
        $ echo "node and wasmtime are required on this runner but missing"; false
        *
    }
    exit script;
}
if has_targets == "0" {
    if require == "1" {
        $ echo "the wasm32-* targets are required on this runner but missing"; false
        *
    }
    exit script;
}

cd "wasm/rust";
defer {
    $ cargo clean --quiet
}
$ cargo build --target wasm32-unknown-unknown --target wasm32-wasip1 --target wasm32-wasip2 --quiet 2>&1
*

# wasm32-unknown-unknown via node (env::write shim + atexit collection)
$ node ../js/loader.mjs target/wasm32-unknown-unknown/debug/wasm_rust.wasm
unordered {
    ! ctor_slices:
    ! 0: Hello, world!
    ! 1: These slices were loaded from the custom section!
    ! MOVABLE_LINK_SECTION: [10, 20, 30, 40]
}
! ctor
! start
! test_link_section
! DRIVER: driver
! driver
! dtor

# wasm32-wasip1 via wasmtime
$ wasmtime run target/wasm32-wasip1/debug/wasm_rust.wasm 2>&1
unordered {
    ! ctor_slices:
    ! 0: Hello, world!
    ! 1: These slices were loaded from the custom section!
    ! MOVABLE_LINK_SECTION: [10, 20, 30, 40]
}
! ctor
! start
! test_link_section
! DRIVER: driver
! driver
! dtor

# wasm32-wasip1 via node (WASI preview1)
$ node --no-warnings ../js/loader-wasi.mjs target/wasm32-wasip1/debug/wasm_rust.wasm
unordered {
    ! ctor_slices:
    ! 0: Hello, world!
    ! 1: These slices were loaded from the custom section!
    ! MOVABLE_LINK_SECTION: [10, 20, 30, 40]
}
! ctor
! start
! test_link_section
! DRIVER: driver
! driver
! dtor

# wasm32-wasip2 via wasmtime
$ wasmtime run target/wasm32-wasip2/debug/wasm_rust.wasm 2>&1
unordered {
    ! ctor_slices:
    ! 0: Hello, world!
    ! 1: These slices were loaded from the custom section!
    ! MOVABLE_LINK_SECTION: [10, 20, 30, 40]
}
! ctor
! start
! test_link_section
! DRIVER: driver
! driver
! dtor
"#
);

// Regression for https://github.com/mmastrac/linktime/issues/488
//
// Two identical items are submitted from a dependency crate and the whole
// workspace is built with fat LTO. The old WASM implementation counted items by
// the byte length of a per-item marker custom section, which fat LTO folded down
// to a single byte.
//
// Only needs `node` and the `wasm32-unknown-unknown` target (no `wasmtime`), so
// it probes separately from `runtime`. A build or run failure fails the test —
// do not add an `%EXIT any` / `choice` arm that swallows it, or the regression
// stops being checked.
crok!(
    fat_lto,
    r#"
set RUSTFLAGS "";

$ sh -c '[ -n "${CI:-}" ] && echo 1 || echo 0'
%SET is_ci
*
$ sh -c '[ -n "${WASM_PLATFORM:-}" ] && echo 1 || echo 0'
%SET require
*
if is_ci == "1" {
    if require == "0" {
        exit script;
    }
}
$ command -v node >/dev/null 2>&1 && echo 1 || echo 0
%SET has_node
*
$ rustup target list --installed 2>/dev/null | grep -q '^wasm32-unknown-unknown$' && echo 1 || echo 0
%SET has_target
*
if has_node == "0" {
    if require == "1" {
        $ echo "node is required on this runner but missing"; false
        *
    }
    exit script;
}
if has_target == "0" {
    if require == "1" {
        $ echo "the wasm32-unknown-unknown target is required on this runner but missing"; false
        *
    }
    exit script;
}

cd "wasm/fat-lto";
defer {
    $ cargo clean --quiet
}
# Build the fat-LTO app and run it. No `%EXIT any`: a build or run failure must
# fail the test (the target probe above already handles the skip case). The `*`
# consumes cargo's `Compiling ...` stderr.
$ cargo build -p app --release --target wasm32-unknown-unknown 2>&1 && node run.mjs target/wasm32-unknown-unknown/release/app.wasm 2>&1
*
! items_len=2 items_sum=14
"#
);
