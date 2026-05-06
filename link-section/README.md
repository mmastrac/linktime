![Build Status](https://github.com/mmastrac/linktime/actions/workflows/rust.yml/badge.svg)

The crate is part of the [`linktime`](https://crates.io/crates/linktime) project.

| crate          | docs                                                                               | version                                                                                                 |
| -------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `linktime`     | [![docs.rs](https://docs.rs/linktime/badge.svg)](https://docs.rs/linktime)         | [![crates.io](https://img.shields.io/crates/v/linktime.svg)](https://crates.io/crates/linktime)       |
| `ctor`         | [![docs.rs](https://docs.rs/ctor/badge.svg)](https://docs.rs/ctor)                 | [![crates.io](https://img.shields.io/crates/v/ctor.svg)](https://crates.io/crates/ctor)                 |
| `dtor`         | [![docs.rs](https://docs.rs/dtor/badge.svg)](https://docs.rs/dtor)                 | [![crates.io](https://img.shields.io/crates/v/dtor.svg)](https://crates.io/crates/dtor)                 |
| `link-section` | [![docs.rs](https://docs.rs/link-section/badge.svg)](https://docs.rs/link-section) | [![crates.io](https://img.shields.io/crates/v/link-section.svg)](https://crates.io/crates/link-section) |
# link-section
A crate for defining link sections in Rust.

Sections are defined using the `#[section]` macro. This creates an associated
`data` and `text` section, and items decorated with the `#[in_section]` macro
are placed into the associated section.

## Platform Support

| Platform                 | Support                                                                                       |
| ------------------------ | --------------------------------------------------------------------------------------------- |
| Linux                    | ✅ Supported, uses orphan section handling (§1)                                               |
| \*BSD                    | ✅ Supported, uses orphan section handling (§1)                                               |
| macOS                    | ✅ Fully supported                                                                            |
| Windows                  | ✅ Fully supported                                                                            |
| WASM                     | ✅ Fully supported (§2) (§3) |
| Other LLVM/GCC platforms | ✅ Supported, uses orphan section handling (§1)                                               |

(§1) Orphan section handling is a feature of the linker that allows sections to
be defined without a pre-defined name.

(§2) WASM requires `const` items, and uses `ctor`-like initialization to copy
data to a contiguous section. To access link-section slices in WASM in
`#[ctor]` functions, make sure to use at least `#[ctor(priority = 1)]`.

(§3) Host environment support (by calling the exported `register_link_section`
function) is required to register each section with the runtime.

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

See [this blog
post](https://devblogs.microsoft.com/oldnewthing/20181107-00/?p=100155) and
[this blog
post](https://devblogs.microsoft.com/oldnewthing/20181108-00/?p=100165) for more
details about the alphabetical sorting rule.

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

## Usage

Create an untyped section using the `#[section]` macro that keeps related items
in close proximity:

```rust
use link_section::{in_section, section};

#[section]
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

    #[section]
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
