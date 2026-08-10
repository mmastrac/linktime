# linktime

Cross-platform libraries for link-time initialization, finalization and
collection in Rust.

[![GitHub](https://img.shields.io/badge/repo-github-blue)](https://github.com/mmastrac/linktime) [![Crates.io License](https://img.shields.io/crates/l/link-section)](https://crates.io/crates/link-section) [![Build Status](https://github.com/mmastrac/linktime/actions/workflows/rust.yml/badge.svg)](https://github.com/mmastrac/linktime/actions/workflows/rust.yml) 

| crate | |
| --- | --- |
| `linktime`<br>[![docs.rs](https://docs.rs/linktime/badge.svg)](https://docs.rs/linktime) [![crates.io](https://img.shields.io/crates/v/linktime.svg)](https://crates.io/crates/linktime) | Convenience crate for `ctor`, `dtor` and `link-section` |
| `ctor`<br>[![docs.rs](https://docs.rs/ctor/badge.svg)](https://docs.rs/ctor) [![crates.io](https://img.shields.io/crates/v/ctor.svg)](https://crates.io/crates/ctor) | Module initialization functions before main |
| `dtor`<br>[![docs.rs](https://docs.rs/dtor/badge.svg)](https://docs.rs/dtor) [![crates.io](https://img.shields.io/crates/v/dtor.svg)](https://crates.io/crates/dtor) | Module shutdown functions before main |
| `link-section`<br>[![docs.rs](https://docs.rs/link-section/badge.svg)](https://docs.rs/link-section) [![crates.io](https://img.shields.io/crates/v/link-section.svg)](https://crates.io/crates/link-section) | Linker-managed typed (slices) and untyped sections |
| `scattered-collect`<br>[![docs.rs](https://docs.rs/scattered-collect/badge.svg)](https://docs.rs/scattered-collect) [![crates.io](https://img.shields.io/crates/v/scattered-collect.svg)](https://crates.io/crates/scattered-collect) | Linker-managed collections: slices, sorted slices, maps |

## Crates

The `linktime` project comprises three crates, and the top-level `linktime`
crate aggregates them all.

Pick-and-choose, or import the top-level crate to get all three.

## [`ctor`](ctor/)

Module initialization functions for Rust (like `__attribute__((constructor))` in
C/C++).

Run code before `main` to initialize data, external resources, or other state.

```toml
[dependencies]
linktime = { version = "...", features = ["ctor"] }  # note: already enabled by default
# or
ctor = "..."
```

```rust
use linktime::ctor; // or ctor::ctor
use libc_print::*;

#[ctor(unsafe)]
fn foo() {
    libc_println!("Life before main!");
}
```

## [`dtor`](dtor/)

Module shutdown functions for Rust (like `__attribute__((destructor))`).

Run code after `main` to clean up resources, or perform other final operations.

```toml
[dependencies]
linktime = { version = "...", features = ["dtor"] }  # note: already enabled by default
# or
dtor = "..."
```

```rust
use linktime::dtor; // or dtor::dtor
use libc_print::*;

#[dtor(unsafe)]
fn foo() {
    libc_println!("Life after main!");
}
```

## [`link-section`](link-section/)

Typed and untyped link section support for Rust.

Collect related items from an entire linked binary into a single link section.

```toml
[dependencies]
linktime = { version = "...", features = ["link-section"] }  # note: already enabled by default
# or
link-section = "..."
```

```rust
use linktime::link_section::{section, in_section, TypedSection};
use linktime::ctor;
use libc_print::*;

#[section(typed)]
static FOO: TypedSection<fn()>;

#[in_section(FOO)]
fn foo() {
    libc_println!("Hello, world!");
}

#[ctor(unsafe)]
fn print_numbers() {
    for f in FOO {
        f();
    }
}
```

## [`scattered-collect`](scattered-collect/)

A crate for defining zero-allocation,linker-managed scattered collections in
Rust.

- `ScatteredIterable`: A collection of items that are available only via
- `ScatteredSlice`: A collection of sized items that collected into a slice in
  an arbitrary order.
- `ScatteredSortedSlice`: A collection of items that are available via slice,
  in sorted order.
- `ScatteredReferencedSlice`: A collection of items collected into a slice
  (link order).
- `ScatteredSortedReferencedSlice`: A collection of sized items that are
  available both via sorted slice and via reference at the declaration site.
- `ScatteredMap`: A collection of key-value pairs that are available via
  slice, as well as indexed by key.

```rust,ignore
use scattered_collect::{gather, scatter, slice::ScatteredSlice};

#[gather]
static SLICE_PLUGINS: ScatteredSlice<&'static str>;

#[scatter(SLICE_PLUGINS)]
const _: &'static str = "json";

#[scatter(SLICE_PLUGINS)]
const _: &'static str = "yaml";

fn main() {
    assert_eq!(SLICE_PLUGINS.len(), 2);
    assert!(SLICE_PLUGINS.contains(&"json"));
}
```

## UEFI

UEFI is fully supported, with one caveat: firmware never runs `.init_array` or
`.fini_array`, and there is no `atexit`. Constructors and destructors are
instead collected into linker sections and run explicitly, so a UEFI binary must
drive its own lifecycle:

```rust,ignore
#[export_name = "efi_main"]
extern "efiapi" fn efi_main(_handle: *mut c_void, _st: *mut c_void) -> usize {
    // Constructors don't run on their own here.
    unsafe { linktime::run_constructors() };

    // ... application ...

    // Neither do destructors.
    unsafe { linktime::run_destructors() };
    0
}
```

This is the only UEFI-specific step. `#[ctor]`, `#[dtor]`, and `#[link_section]`
collection otherwise behave as on any other platform, and it works identically
whether the binary is built with `std` (`fn main`) or `#![no_std]`. `#[ctor]`
priorities are honored (constructors run in ascending priority order).

## Contributing

Contributions are welcome!

## License

These projects are dual-licensed under the Apache License, Version 2.0 and the
MIT License.
