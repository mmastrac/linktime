Shutdown functions for Rust (like `__attribute__((destructor))` in C/C++) for
Linux, macOS, Windows, mobile (iOS/Android), WASM, BSD/BSD-likes and many other
platforms.

```rust
use dtor::dtor;
use libc_print::*;

#[dtor(unsafe)]
fn foo() {
    libc_println!("Life after main!");
}
```

# Examples

Print a message at shutdown time.

```rust
# use dtor::dtor;
# use libc_print::*;
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
| WASM                       | No                                        | Yes            | No                   |
| AIX                        | "Kind of" <sup><sup>🔵</sup></sup>        | Yes            | Yes                  |
| Other POSIX-like platforms | `.fini_array`/`.dtors`                    | Yes (`atexit`) | Yes (`__cxa_atexit`) |

Notes:

- <sup><sup>🍎</sup></sup> Not recommended. Apple platforms no longer call
  `mod_term_func` functions.
- <sup><sup>🪟</sup></sup> Not recommended. Windows platforms may not reliably
  call functions in link sections, unless a binary is built with a static CRT.
- <sup><sup>🔵</sup></sup> Link sections are not supported on AIX, but the
  platform calls functions with the prefix `__sinit` and `__sterm` at startup
  and shutdown respectively. `__sterm`-prefixed functions are used when the
  method is specified as `linker`.

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
  (Windows) so the dtor runs when the module unloads. Unsupported on WASM.
- `at_binary_exit`: Register to run at process exit (unsupported on Windows).
- `linker`: Register using the platform's linker mechanism (`link_section` on
  all platforms with the exception of `export_name_prefix` on AIX). Unsupported
  on Apple platforms.

Default:

- Apple and Windows default to `at_module_exit`
- WASM defaults to `at_binary_exit` (note that you will need to provide your own
  atexit implementation for `wasm32-unknown-unknown`)
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
/// and `export_name_prefix` on AIX.
/// 
/// Platform note: this will fail to compile on Apple platforms.
# #[cfg(not(target_vendor = "apple"))]
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
