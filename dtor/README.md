![Build Status](https://github.com/mmastrac/linktime/actions/workflows/rust.yml/badge.svg)

The crate is part of the [`linktime`](https://crates.io/crates/linktime) project.

| crate          | docs                                                                               | version                                                                                                 |
| -------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `linktime`     | [![docs.rs](https://docs.rs/linktime/badge.svg)](https://docs.rs/linktime)         | [![crates.io](https://img.shields.io/crates/v/linktime.svg)](https://crates.io/crates/linktime)       |
| `ctor`         | [![docs.rs](https://docs.rs/ctor/badge.svg)](https://docs.rs/ctor)                 | [![crates.io](https://img.shields.io/crates/v/ctor.svg)](https://crates.io/crates/ctor)                 |
| `dtor`         | [![docs.rs](https://docs.rs/dtor/badge.svg)](https://docs.rs/dtor)                 | [![crates.io](https://img.shields.io/crates/v/dtor.svg)](https://crates.io/crates/dtor)                 |
| `link-section` | [![docs.rs](https://docs.rs/link-section/badge.svg)](https://docs.rs/link-section) | [![crates.io](https://img.shields.io/crates/v/link-section.svg)](https://crates.io/crates/link-section) |
# dtor
Shutdown functions for Rust (like `__attribute__((destructor))` in C/C++) for
Linux, OSX, Windows, mobile (iOS/Android), WASM, BSD/BSD-likes and many other
platforms.

```rust
use dtor::dtor;

#[dtor(unsafe)]
fn foo() {
    println!("Life after main!");
}
```

# Examples

Print a message at shutdown time.

```rust
#[dtor(unsafe)]
fn shutdown() {
    // Using println! or eprintln! here may panic as Rust may have
    // shut down some stdlib services at this time.
    libc_println!("Shutting down!");
}
```

# Platform Support

| Platform                   | Link Section                              | at_binary_exit | at_module_exit       |
| -------------------------- | ----------------------------------------- | -------------- | -------------------- |
| Linux                      | `.fini_array`                             | Yes (`atexit`) | Yes (`__cxa_atexit`) |
| MacOS                      | `.mod_term_func` <sup><sup>🍎</sup></sup> | Yes (`atexit`) | Yes (`__cxa_atexit`) |
| Windows                    | `.CRT$XPU` <sup><sup>🪟</sup></sup>       | No             | Yes (`atexit`)       |
| AIX                        | No <sup><sup>🔵</sup></sup>               | Yes            | Yes                  |
| Other POSIX-like platforms | `.fini_array`/`.dtors`                    | Yes (`atexit`) | Yes (`__cxa_atexit`) |

Notes:

- <sup><sup>🍎</sup></sup> Not recommended. Apple platforms no longer call
  `mod_term_func` functions.
- <sup><sup>🪟</sup></sup> Not recommended. Windows platforms may not reliably
  call functions in link sections, unless a binary is built with a static CRT.
- <sup><sup>🔵</sup></sup> Link sections are not supported on AIX, but the
  platform calls functions with the prefix `__sinit` and `__sterm` at startup
  and shutdown respectively.

# Shutdown Method (`#[dtor(method = ...)]`)

The `#[dtor]` macro supports multiple registration strategies via
`#[dtor(method = ...)]`. The best choice is platform-dependent:

- `#[dtor]` (no method specified): Use the platform's most reliable method:
  `at_module_exit` on Windows and Apple platforms, and `linker` on others.
- `unload`: Run on _module unload_ (library unload or process exit) using the
  platform's default unload method.
- `term`: Run on _process termination only_ using the platform's default
  termination method. Not recommended: code may be unloaded before the dtor
  runs.
- `at_module_exit`: Register using `__cxa_atexit` (non-Windows) or `atexit`
  (Windows) so the dtor runs when the module unloads.
- `at_binary_exit`: Register to run at process exit (unsupported on Windows).
- `linker`: Register using the platform's linker mechanism (`link_section` on
  all platforms with the exception of `export_name_prefix` on AIX). Unsupported
  on Apple platforms.

Default:

- Apple and Windows default to `at_module_exit`
- Most other platforms default to `linker`

Examples:

```rust
use dtor::dtor;
/// Use `at_module_exit` on all platforms
#[dtor(unsafe, method = at_module_exit)]
fn shutdown() {}
```

```rust
use dtor::dtor;

/// Use `link_section` with a section name of `.dtors` on most platforms,
/// and `export_name_prefix` on AIX
#[dtor(unsafe, method = linker, link_section = ".dtors")]
fn shutdown() {}
```

# Warnings

Rust's philosophy is that nothing happens before or after main and this library
explicitly subverts that. The code that runs in the `ctor` and `dtor` functions
should be careful to limit itself to `libc` functions and code that does not
rely on Rust's stdlib services.

See [`::life_before_main`](crate::life_before_main) for more information.

# Under the Hood

The `#[dtor]` macro effectively creates a constructor that calls `libc::atexit`
with the provided function, i.e. roughly equivalent to:

```rust,ignore
#[ctor]
fn dtor_atexit() {
    libc::atexit(dtor);
}
```
# Crate Features

| Cargo feature | Description |
| --- | --- |
| `no_warn_on_missing_unsafe` |  Do not warn when a ctor or dtor is missing the `unsafe` keyword. |
| `proc_macro` |  Enable support for the proc-macro `#[dtor]` attribute. The declarative form (`dtor!(...)`) is always available. It is recommended that crates re-exporting the `dtor` macro disable this feature and only use the declarative form. |
| `std` |  Enable support for the standard library. |
| `used_linker` |  Applies `used(linker)` to all `dtor`-generated functions. Requires nightly and `feature(used_with_arg)`. |

# Macro Attributes

<table><tr><th>Attribute</th><th>Description</th></tr>
<tr><td><code>anonymous</code></td><td>

 Make the ctor function anonymous.


</td></tr>
<tr><td><code>crate_path = $path : pat</code></td><td>

 Specify a custom crate path for the `dtor` crate. Used when re-exporting the dtor macro.


</td></tr>
<tr><td><code>ctor(export_name_prefix = $ctor_export_name_prefix_str : literal)</code></td><td>

 Specify a custom export name prefix for the constructor function.

 If specified, an export with the given prefix will be generated in the form:

 `<prefix>_<unique_id>`


</td></tr>
<tr><td><code>ctor(link_section = $ctor_link_section_name : literal)</code></td><td>

 Place the initialization function pointer in a custom link section.


</td></tr>
<tr><td><code>export_name_prefix = $export_name_prefix_str : literal</code></td><td>

 Specify a custom export name prefix for the destructor function.

 If specified, an export with the given prefix will be generated in the form:

 `<prefix>_<unique_id>`


</td></tr>
<tr><td><code>link_section = $section : literal</code></td><td>

 Place the destructor function pointer in a custom link section.


</td></tr>
<tr><td><code>method = $method_id : ident</code></td><td>

 Specify the dtor method.

  - `term`: Run the dtor on binary termination using the platform's
    [default_term_method](#default_term_method). Not recommended as code
    may be unloaded before the dtor is called.
  - `unload`: Run the dtor on module unload (library or binary) using the
    platform's [default_unload_method](#default_unload_method).
  - `at_module_exit`: Run the dtor using the platform's
    [`at_module_exit`][at_module_exit] (`__cxa_atexit` on all platforms
    other than Windows, `atexit` on Windows).
  - `at_binary_exit`: Run the dtor using the platform's
    [`at_binary_exit`][at_binary_exit] (unsupported on Windows
    platforms).
  - `linker`: Register the dtor using the platform's
    [link_section](#link_section) or
    [export_name_prefix](#export_name_prefix) (unsupported on Apple
    platforms).

 [at_module_exit]: crate::native::at_module_exit
 [at_binary_exit]: crate::native::at_binary_exit


</td></tr>
<tr><td><code>unsafe</code></td><td>

 Marks a ctor/dtor as unsafe.


</td></tr>
<tr><td><code>used(linker)</code></td><td>

 Mark generated functions for this `dtor` as `used(linker)`. Requires nightly and `feature(used_with_arg)`.


</td></tr>
</table>

# Defaults

## `ctor_export_name_prefix`

 ```rust
#[cfg(target_os = "aix")]
ctor_export_name_prefix = "__sinit80000000"

 // default
ctor_export_name_prefix = ()
 ```

## `ctor_link_section`

 ```rust
#[cfg(target_vendor = "apple")]
ctor_link_section = "__DATA,__mod_init_func,mod_init_funcs"

#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd",
target_os = "netbsd", target_os = "openbsd", target_os = "dragonfly",
target_os = "illumos", target_os = "haiku", target_os = "vxworks", target_os =
"nto", target_family = "wasm"))]
ctor_link_section = ".init_array"

#[cfg(target_os = "none")]
ctor_link_section = ".init_array"

#[cfg(target_arch = "xtensa")]
ctor_link_section = ".ctors"

#[cfg(all(target_vendor = "pc", any(target_env = "gnu", target_env = "msvc")))]
ctor_link_section = ".CRT$XCU"

#[cfg(all(target_vendor = "pc", not(any(target_env = "gnu", target_env = "msvc"))))]
ctor_link_section = ".ctors"

#[cfg(all(target_os = "aix"))]
ctor_link_section = ()

 // default
ctor_link_section = (compile_error! ("Unsupported target for #[ctor]"))
 ```

## `default_term_method`

 ```rust
#[cfg(target_vendor = "pc")]
default_term_method = at_module_exit

 // default
default_term_method = at_binary_exit
 ```

## `default_unload_method`

 ```rust
 // default
default_unload_method = at_module_exit
 ```

## `export_name_prefix`

 ```rust
#[cfg(target_os = "aix")]
export_name_prefix = "__sterm80000000"

 // default
export_name_prefix = ()
 ```

## `link_section`

 ```rust
#[cfg(target_vendor = "apple")]
link_section = "__DATA,__mod_term_func,mod_term_funcs"

#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd",
target_os = "netbsd", target_os = "openbsd", target_os = "dragonfly",
target_os = "illumos", target_os = "haiku", target_os = "vxworks", target_os =
"nto", target_family = "wasm"))]
link_section = ".fini_array"

#[cfg(target_os = "none")]
link_section = ".fini_array"

#[cfg(target_arch = "xtensa")]
link_section = ".dtors"

#[cfg(all(target_vendor = "pc", any(target_env = "gnu", target_env = "msvc")))]
link_section = ".CRT$XPU"

#[cfg(all(target_vendor = "pc", not(any(target_env = "gnu", target_env = "msvc"))))]
link_section = ".dtors"

#[cfg(all(target_os = "aix"))]
link_section = ()

 // default
link_section = (compile_error! ("Unsupported target for #[dtor]"))
 ```

## `method`

 ```rust
#[cfg(target_vendor = "apple")]
method = at_module_exit

#[cfg(target_vendor = "pc")]
method = at_module_exit

 // default
method = linker
 ```
