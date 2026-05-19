![Build Status](https://github.com/mmastrac/linktime/actions/workflows/rust.yml/badge.svg)

The crate is part of the [`linktime`](https://crates.io/crates/linktime) project.

| crate               |                                                         | docs                                                                                         | version                                                                                                           |
| ------------------- | ------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `linktime`          | Convenience crate for `ctor`, `dtor` and `link-section` | [![docs.rs](https://docs.rs/linktime/badge.svg)](https://docs.rs/linktime)                   | [![crates.io](https://img.shields.io/crates/v/linktime.svg)](https://crates.io/crates/linktime)                   |
| `ctor`              | Module initialization functions before main             | [![docs.rs](https://docs.rs/ctor/badge.svg)](https://docs.rs/ctor)                           | [![crates.io](https://img.shields.io/crates/v/ctor.svg)](https://crates.io/crates/ctor)                           |
| `dtor`              | Module shutdown functions before main                   | [![docs.rs](https://docs.rs/dtor/badge.svg)](https://docs.rs/dtor)                           | [![crates.io](https://img.shields.io/crates/v/dtor.svg)](https://crates.io/crates/dtor)                           |
| `link-section`      | Linker-managed typed (slices) and untyped sections      | [![docs.rs](https://docs.rs/link-section/badge.svg)](https://docs.rs/link-section)           | [![crates.io](https://img.shields.io/crates/v/link-section.svg)](https://crates.io/crates/link-section)           |
| `scattered-collect` | Linker-managed collections: slices, sorted slices, maps | [![docs.rs](https://docs.rs/scattered-collect/badge.svg)](https://docs.rs/scattered-collect) | [![crates.io](https://img.shields.io/crates/v/scattered-collect.svg)](https://crates.io/crates/scattered-collect) |
# link-section
A crate for defining linker-backed sections in Rust.

`link-section` provides two attributes:

- `#[section(...)]` defines a section handle. The handle is a `static` item used
  to inspect the section at runtime, usually as a slice.
- `#[in_section(SECTION)]` submits an item to that section. A submitted item is
  an item annotated with `#[in_section(...)]`; depending on the section kind, it
  may also remain usable directly at the submission site.

Together, these attributes let separately-declared items be collected into one
linker section and accessed through a single section handle.

## Section Kinds

There are five section kinds:

- `untyped`: Collects related code or data in one linker section without
  exposing a typed slice. This is useful for co-location, phase-specific code,
  or platform-specific section placement.
- `typed`: Stores values of one type and exposes them as an immutable slice.
- `mutable`: Stores values of one type and exposes them as a mutable slice.
- `reference`: Stores values of one type, exposes them as an immutable slice,
  and also lets each submitted item be used as a reference at its submission
  site.
- `movable`: Stores values of one type and exposes them as a mutable slice, and
  also lets each submitted item be used as a reference at its submission site.
  The entire section is available as a mutable slice, and items may be reordered
  during startup initialization (see [`TypedMovableSection`] for more details).

| Section Kind | Immutable Slice | Mutable Slice | `const` Items | `static` / Reference Items |
| ------------ | --------------- | ------------- | ------------- | -------------------------- |
| `untyped`    | ❌              | ❌            | ✅            | ✅                         |
| `typed`      | ✅              | ❌            | ✅            | ⚠️                         |
| `mutable`    | ✅              | ✅            | ✅            | ❌                         |
| `reference`  | ✅              | ❌            | ✅            | ✅                         |
| `movable`    | ✅              | ✅            | ❌            | ✅                         |

⚠️ Native targets support `static` submissions for `typed` sections; WASM uses
`const` submissions only.

## Submitting Items

Items are submitted with `#[in_section(SECTION)]`.

A `const` submission copies the value into the section. The original constant
remains usable as a normal Rust constant, and the section receives its own
stored copy.

```rust
#[in_section(MY_SECTION)]
pub const ITEM: MyType = MyType::new();
```

A `static` submission stores the `static` directly in the section. References to
the `static` and references obtained from the section slice point at the same
underlying object. `static` submissions are supported for typed sections on
native targets and for reference sections.

```rust
#[in_section(MY_SECTION)]
pub static ITEM: MyType = MyType::new();
```

A `fn` submitted to a typed section is stored as a function pointer. The
function body itself is not placed into the typed data section.

```rust
#[section(typed)]
pub static FUNCTIONS: link_section::TypedSection<fn()>;

#[in_section(FUNCTIONS)]
pub fn callback() {
    // ...
}
```

## Platform Support

| Platform                 | Support                                         |
| ------------------------ | ----------------------------------------------- |
| Linux                    | ✅ Supported, uses orphan section handling (§1) |
| \*BSD                    | ✅ Supported, uses orphan section handling (§1) |
| macOS                    | ✅ Fully supported                              |
| Windows                  | ✅ Fully supported                              |
| WASM                     | ✅ Fully supported, via emulation (§2) (§3)     |
| AIX                      | ✅ Supported (§4)                               |
| Other LLVM/GCC platforms | ✅ Supported, uses orphan section handling (§1) |

(§1) Orphan section handling is a feature of the linker that allows sections to
be defined without a pre-defined name.

(§2) WASM requires `const` items, and uses `ctor`-like initialization to copy
data to a contiguous section. To access link-section slices in WASM in `#[ctor]`
functions, make sure to use at least `#[ctor(priority = 1)]`.

(§3) Host environment support (by calling the exported `read_custom_section`
function) is required to register each section with the runtime.

(§4) AIX requires `-C link-arg=-bdbg:namedsects:ss` which enables functionality
similar to LLVM/GCC's orphan section handling.

## Platform Details

Each platform has a slightly different implementation of section control.

### Linux and other LLVM/GCC platforms

- Has start/end symbols: ✅ (C-compatible names only)
- Supports linker sorting: ❌

On Linux and other LLVM/GCC platforms, the linker supports orphan sections,
which allow sections to be defined without a pre-defined name. These sections
are emitted as if they were r/w `.data`. For sections with C-compatible names,
the linker will emit start/end symbols for the section.

Orphan sections are not sorted via numeric suffix (e.g.: `SECTION.1`,
`SECTION.2`, etc.) with the default linker script.

### macOS

- Has start/end symbols: ✅
- Supports linker sorting: ❌

On macOS, sections are configured via `__DATA` or `__TEXT` prefix and option
suffixes (`regular`, `no_dead_strip`, etc.). The linker emits start and stop
symbols, but Rust requires a (somewhat-stable) `\x01` prefix to avoid mangling
the section name. macOS does not support ordering in the linker.

### Windows

- Has start/end symbols: ❌
- Supports linker sorting: ✅

On Windows, the linker does not emit start/end symbols, but all sections with a
common prefix are automatically sorted by suffix, allowing us to use suffixes to
control placement of start/stop symbols that we emit.

See
[this blog post](https://devblogs.microsoft.com/oldnewthing/20181107-00/?p=100155)
and
[this blog post](https://devblogs.microsoft.com/oldnewthing/20181108-00/?p=100165)
for more details about the alphabetical sorting rule.

### WASM

- Has start/end symbols: ❌
- Supports linker sorting: ❌

On WASM platforms, Rust emits data into custom sections which do not support
ordering, and are stored out-of-band. The host environment is responsible for
registering this out-of-band section with this library as this data is not
accessible by the WASM runtime.

Normally, WASM does not support placing arbitrary data in link sections - only
non-pointer data is supported. However, the WASM support uses `const` items and
pre-main construction functions to copy each entry into a contiguous section
allocated at startup. The number of items in a link-section is computed by
generating a custom data section containing one byte per item.

The WASM support expects a function in the module's environment with the
following signature and functionality. The wasm import only passes the four
`usize` / pointer parameters; the embedder should close over
`WebAssembly.Module` and `WebAssembly.Memory` from compile/instantiate when
installing the import.

```js
/**
 * Support function for `link-section` crate.
 */
export function readCustomSection(
  wasmModule: WebAssembly.Module,
  wasmInstance: WebAssembly.Instance,
  namePtr: number,
  nameLength: number,
  targetPtr: number,
  targetLength: number,
): number {
    const memory = wasmInstance.exports.memory as WebAssembly.Memory;
    const nameBytes = new Uint8Array(memory.buffer, namePtr, nameLength);
    const sectionName = new TextDecoder().decode(nameBytes);

    const sections = WebAssembly.Module.customSections(wasmModule, sectionName);
    if (sections.length === 0) {
        return 0;
    }

    const section = sections[0];
    const need = section.byteLength;
    if (targetLength < need) {
        return need;
    }

    new Uint8Array(memory.buffer, targetPtr, need).set(new Uint8Array(section));
    return need;
}
```

### AIX

- Has start/end symbols: ✅
- Supports linker sorting: ❌

AIX maps Rust's `#[link_section]` to `csect`s (Control Sections), which act like
subsections of the larger `.text` and `.data` sections
<sup>[↳](https://www.ibm.com/docs/kk/aix/7.2.0?topic=program-understanding-programming-toc)</sup>.
A `csect` is the smallest, indivisible unit of code or data.

By default, AIX does not have section start/stop symbols, but the most recent
versions of the linker added a new `-bdbg:namedsects:ss` flag which enables
section start/stop symbols
<sup>[↳](https://reviews.llvm.org/D124857?id=427067)</sup>.

This flag can be set with `-C link-arg=-bdbg:namedsects:ss` (or by upgrading to
a recent Rust version that sets this automatically
<sup>[↳](https://rust.googlesource.com/rust/+/ad582a586550bf2c72e963939f61a71df1af7c0c%5E%21/#F0)</sup>
to support link sections.

The linker will report an error like this if the start/stop symbols are not
found:

```text
= note: ld: 0711-317 ERROR: Undefined symbol: __start__data_link_section_DATABASES
        ld: 0711-317 ERROR: Undefined symbol: __stop__data_link_section_DATABASES
        ld: 0711-345 Use the -bloadmap or -bnoquiet option to obtain more information.
```

For debugging AIX link-section issues, `-C link-arg=-bmap:[path]/linker.out` and
`-C link-arg=-bnoquiet` may also be useful.

AIX supports a special mode to strip (`strip -r`) that preserves structural
symbols like `csect`s and exports. A future version of `link-section` may add
support for loading `csect` bounds from the binary's symbol table.

```toml
[target.powerpc64-ibm-aix]
rustflags = [
    "-C", "link-arg=-bdbg:namedsects:ss",   # required
    "-C", "link-arg=-bmap:linker.out",      # for debugging
    "-C", "link-arg=-bnoquiet",             # for debugging
]
```

## Typed Sections

Typed sections provide a section where all items are of a specific, sized type.
The typed section may be accessed as a slice of the type at zero cost if
desired.

A typed section can be created from either `static` or `const` items.

For `const` items: a copy of the `const` is materialized at link time, while the
constant itself remains available for use as a constant in `const` contexts.

For `static` items: the static is stored directly in the link section.

`fn` items are special-cased and stored as function pointers in the typed
section.

## Exclusive Access

Mutable sections (ie: [`TypedMutableSection`] and [`TypedMovableSection`])
require exclusive access to the section's memory while calling
[`TypedMutableSection::as_mut_slice`] or [`TypedMovableSection::as_mut_slice`].

This is normally satisfied only during pre-`main` initialization (for example
inside a `#[ctor]`). After `main`, the caller must guarantee no concurrent reads
or writes from other threads and no active Rust references into the section.

It is highly recommended not to access the mutable references after `main` has
started.

## Usage

Create an untyped section using the `#[section]` macro that keeps related items
in close proximity:

```rust
use link_section::{in_section, section};

#[section(untyped)]
pub static CODE_SECTION: link_section::Section;

#[in_section(CODE_SECTION)]
pub fn link_section_function() {
    println!("link_section_function");
}
```

Create a typed section using the `#[section]` macro that stores items of a
specific, sized type from `static` or `const` items:

```rust
mod my_registry {
    use link_section::{in_section, section};

    pub struct MyStruct {
        name: &'static str,
    }

    #[section(typed)]
    pub static MY_REGISTRY: link_section::TypedSection<MyStruct>;

    // Registers a `const` item.
    mod register_a_constant {
        use super::*;

        // A copy of this constant is registered in the link section.
        #[in_section(MY_REGISTRY)]
        pub const LINKED_MY_STRUCT: MyStruct = MyStruct { name: "my_struct" };
    }

    // Registers a `static` item.
    mod register_a_static {
        use super::*;

        // This static lives directly in the link section.
        #[in_section(MY_REGISTRY)]
        pub static LINKED_MY_STRUCT: MyStruct = MyStruct { name: "my_struct_2" };
    }
}
```

## Inspiration

`link-section` was originally inspired by the `linkme` project.
