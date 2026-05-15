# linktime

Cross-platform libraries for link-time initialization, finalization and collection in Rust.

![Build Status](https://github.com/mmastrac/linktime/actions/workflows/rust.yml/badge.svg)

| crate          | docs                                                                               | version                                                                                                 |
| -------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `linktime`     | [![docs.rs](https://docs.rs/linktime/badge.svg)](https://docs.rs/linktime)           | [![crates.io](https://img.shields.io/crates/v/linktime.svg)](https://crates.io/crates/linktime)       |
| `ctor`         | [![docs.rs](https://docs.rs/ctor/badge.svg)](https://docs.rs/ctor)                 | [![crates.io](https://img.shields.io/crates/v/ctor.svg)](https://crates.io/crates/ctor)                 |
| `dtor`         | [![docs.rs](https://docs.rs/dtor/badge.svg)](https://docs.rs/dtor)                 | [![crates.io](https://img.shields.io/crates/v/dtor.svg)](https://crates.io/crates/dtor)                 |
| `link-section` | [![docs.rs](https://docs.rs/link-section/badge.svg)](https://docs.rs/link-section) | [![crates.io](https://img.shields.io/crates/v/link-section.svg)](https://crates.io/crates/link-section) |

## Crates

The `linktime` project comprises three crates, and the top-level `linktime`
crate aggregates them all.

Pick-and-choose, or import the top-level crate to get all three.

## [`ctor`](ctor/)

Module initialization functions for Rust (like `__attribute__((constructor))` in C/C++).

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

## Contributing

Contributions are welcome! 

## License

These projects are dual-licensed under the Apache License, Version 2.0 and the MIT License.


