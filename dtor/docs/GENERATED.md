
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
<tr><td><code>crate_path = ::path::to::dtor::crate</code></td><td>

 Specify a custom crate path for the `dtor` crate. Used when re-exporting the dtor macro.


</td></tr>
<tr><td><code>ctor(export_name_prefix = "ctor_")</code></td><td>

 Specify a custom export name prefix for the constructor function.

 If specified, an export with the given prefix will be generated in the form:

 `<prefix>_<unique_id>`


</td></tr>
<tr><td><code>ctor(link_section = ".ctors")</code></td><td>

 Place the initialization function pointer in a custom link section.


</td></tr>
<tr><td><code>export_name_prefix = "ctor_"</code></td><td>

 Specify a custom export name prefix for the destructor function.

 If specified, an export with the given prefix will be generated in the form:

 `<prefix>_<unique_id>`


</td></tr>
<tr><td><code>link_section = ".dtors"</code></td><td>

 Place the destructor function pointer in a custom link section.


</td></tr>
<tr><td><code>method = term|unload|at_module_exit|at_binary_exit|linker</code></td><td>

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
 # #[cfg(false)] {
#[cfg(target_os = "aix")]
 # const _: () = { let
ctor_export_name_prefix = "__sinit80000000"
 # ; };

 // default
ctor_export_name_prefix = ()
 # }
 ```

## `ctor_link_section`

 ```rust
 # #[cfg(false)] {
#[cfg(target_vendor = "apple")]
 # const _: () = { let
ctor_link_section = "__DATA,__mod_init_func,mod_init_funcs"
 # ; };

#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd",
target_os = "netbsd", target_os = "openbsd", target_os = "dragonfly",
target_os = "illumos", target_os = "haiku", target_os = "vxworks", target_os =
"nto", target_family = "wasm"))]
 # const _: () = { let
ctor_link_section = ".init_array"
 # ; };

#[cfg(target_os = "none")]
 # const _: () = { let
ctor_link_section = ".init_array"
 # ; };

#[cfg(target_arch = "xtensa")]
 # const _: () = { let
ctor_link_section = ".ctors"
 # ; };

#[cfg(all(target_vendor = "pc", any(target_env = "gnu", target_env = "msvc")))]
 # const _: () = { let
ctor_link_section = ".CRT$XCU"
 # ; };

#[cfg(all(target_vendor = "pc", not(any(target_env = "gnu", target_env = "msvc"))))]
 # const _: () = { let
ctor_link_section = ".ctors"
 # ; };

#[cfg(all(target_os = "aix"))]
 # const _: () = { let
ctor_link_section = ()
 # ; };

 // default
ctor_link_section = (compile_error! ("Unsupported target for #[ctor]"))
 # }
 ```

## `default_term_method`

 ```rust
 # #[cfg(false)] {
#[cfg(target_vendor = "pc")]
 # const _: () = { let
default_term_method = at_module_exit
 # ; };

 // default
default_term_method = at_binary_exit
 # }
 ```

## `default_unload_method`

 ```rust
 # #[cfg(false)] {
 // default
default_unload_method = at_module_exit
 # }
 ```

## `export_name_prefix`

 ```rust
 # #[cfg(false)] {
#[cfg(target_os = "aix")]
 # const _: () = { let
export_name_prefix = "__sterm80000000"
 # ; };

 // default
export_name_prefix = ()
 # }
 ```

## `link_section`

 ```rust
 # #[cfg(false)] {
#[cfg(target_vendor = "apple")]
 # const _: () = { let
link_section = "__DATA,__mod_term_func,mod_term_funcs"
 # ; };

#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd",
target_os = "netbsd", target_os = "openbsd", target_os = "dragonfly",
target_os = "illumos", target_os = "haiku", target_os = "vxworks", target_os =
"nto", target_family = "wasm"))]
 # const _: () = { let
link_section = ".fini_array"
 # ; };

#[cfg(target_os = "none")]
 # const _: () = { let
link_section = ".fini_array"
 # ; };

#[cfg(target_arch = "xtensa")]
 # const _: () = { let
link_section = ".dtors"
 # ; };

#[cfg(all(target_vendor = "pc", any(target_env = "gnu", target_env = "msvc")))]
 # const _: () = { let
link_section = ".CRT$XPU"
 # ; };

#[cfg(all(target_vendor = "pc", not(any(target_env = "gnu", target_env = "msvc"))))]
 # const _: () = { let
link_section = ".dtors"
 # ; };

#[cfg(all(target_os = "aix"))]
 # const _: () = { let
link_section = ()
 # ; };

 // default
link_section = (compile_error! ("Unsupported target for #[dtor]"))
 # }
 ```

## `method`

 ```rust
 # #[cfg(false)] {
#[cfg(target_vendor = "apple")]
 # const _: () = { let
method = at_module_exit
 # ; };

#[cfg(target_vendor = "pc")]
 # const _: () = { let
method = at_module_exit
 # ; };

 // default
method = linker
 # }
 ```
