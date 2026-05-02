
# Crate Features

| Cargo feature | Description |
| --- | --- |
| `priority_enabled` |  Enable support for the priority parameter. |
| `proc_macro` |  Enable support for the proc-macro `#[ctor]` attribute. The declarative form (`ctor!(...)`) is always available. It is recommended that crates re-exporting the `ctor` macro disable this feature and only use the declarative form. |
| `std` |  Enable support for the standard library. |
| `used_linker` |  Applies `used(linker)` to all `ctor`-generated functions. Requires nightly and `feature(used_with_arg)`. |

# Macro Attributes

<table><tr><th>Attribute</th><th>Description</th></tr>
<tr><td><code>anonymous</code></td><td>

 Do not give the constructor a name in the generated code (allows for
 multiple constructors with the same name). Equivalent to wrapping the
 constructor in an anonymous const (i.e.: `const _ = { ... };`).


</td></tr>
<tr><td><code>body(link_section = ".text.startup")</code></td><td>

 Place the constructor body in a custom link section. By default, this
 uses the appropriate platform-specific link section.

 Co-locating startup functions may improve performance by allowing the binary
 to page them in and out of memory together.


</td></tr>
<tr><td><code>crate_path = ::path::to::ctor::crate</code></td><td>

 The path to the `ctor` crate containing the support macros. If you
 re-export `ctor` items as part of your crate, you can use this to
 redirect the macro’s output to the correct crate.

 Using the declarative [`ctor!`][c] form is
 preferred over this parameter.

 [c]: crate::declarative::ctor!


</td></tr>
<tr><td><code>export_name_prefix = "ctor_"</code></td><td>

 Specify a custom export name prefix for the constructor function.

 If specified, an export with the given prefix will be generated in the form:

 `<prefix><priority>_<unique_id>`


</td></tr>
<tr><td><code>link_section = ".ctors"</code></td><td>

 Place the constructor function pointer in a custom link section. By
 default, this uses the appropriate platform-specific link section.


</td></tr>
<tr><td><code>naked</code></td><td>

 Use the least-possibly mangled version of the linker invocation for this
 constructor. This is not recommended for general use as it may prevent
 authors of binary crates from having low-level control over the order of
 initialization.

 There are no guarantees about the order of execution of constructors
 with this attribute, just that it will be called at some point before
 `main`.

 `naked` constructors are always executed directly by the underlying C
 library and/or dynamic loader.

 `naked` cannot be used with the `priority` attribute.


</td></tr>
<tr><td><code>unsafe</code></td><td>

 Marks a ctor as unsafe. Required.

 The `ctor` crate will warn if there is no unsafe flag in the `ctor`
 annotation. This warning for a missing unsafe keyword can be hidden
 by passing `RUSTFLAGS="--cfg no_fail_on_missing_unsafe"` to Cargo.


</td></tr>
<tr><td><code>priority = N | early | late</code></td><td>

 The priority of the constructor. Higher-`N`-priority constructors are
 run last. `N` must be between 0 and 999 inclusive for ordering
 guarantees (`N` >= 1000 ordering is platform-defined).

 Priority is specified as an isize, string literal, or the identifiers
 `early` or `late`. The integer value will be clamped to a
 platform-defined range (typically 0-65535), while the string value will
 unprocessed.

 Priority is applied as follows:

  - `early` is the default, and is run first (constructors annotated with
    `early` and those with no priority attribute are run in the same
    phase).
  - `N` is run in increasing order, from 0 <= N <= 999.
  - `late` is run last, and will be positioned to run after most
    constructors, even outside the range 0 <= N <= 999.
  - `main` is run, for binary targets.

 Ordering outside of `0 <= N <= 999` is platform-defined with respect to
 the list above, however platforms will order constructors within a given
 length range in ascending order (ie: 10000 will run before 20000).


</td></tr>
<tr><td><code>used(linker)</code></td><td>

 Mark generated functions for this `ctor` as `used(linker)`. Requires nightly and `feature(used_with_arg)`.


</td></tr>
</table>

# Defaults

## `body_link_section`

 ```rust
 # #[cfg(false)] {
#[cfg(target_os = "linux")]
 # const _: () = { let
body_link_section = ".text.startup"
 # ; };

#[cfg(target_os = "android")]
 # const _: () = { let
body_link_section = ".text.startup"
 # ; };

#[cfg(target_os = "freebsd")]
 # const _: () = { let
body_link_section = ".text.startup"
 # ; };

#[cfg(all(target_vendor = "pc", any(target_env = "gnu", target_env = "msvc")))]
 # const _: () = { let
body_link_section = ".text$A"
 # ; };

#[cfg(all(target_vendor = "pc", not(any(target_env = "gnu", target_env = "msvc"))))]
 # const _: () = { let
body_link_section = ".text.startup"
 # ; };

#[cfg(target_vendor = "apple")]
 # const _: () = { let
body_link_section = "__TEXT,__text_startup,regular,pure_instructions"
 # ; };

 // default
body_link_section = ()
 # }
 ```

## `export_name_prefix`

 ```rust
 # #[cfg(false)] {
#[cfg(target_os = "aix")]
 # const _: () = { let
export_name_prefix = "__sinit"
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
link_section = "__DATA,__mod_init_func,mod_init_funcs"
 # ; };

#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd",
target_os = "netbsd", target_os = "openbsd", target_os = "dragonfly",
target_os = "illumos", target_os = "haiku", target_os = "vxworks", target_os =
"nto", target_family = "wasm"))]
 # const _: () = { let
link_section = ".init_array"
 # ; };

#[cfg(target_os = "none")]
 # const _: () = { let
link_section = ".init_array"
 # ; };

#[cfg(target_arch = "xtensa")]
 # const _: () = { let
link_section = ".ctors"
 # ; };

#[cfg(all(target_vendor = "pc", any(target_env = "gnu", target_env = "msvc")))]
 # const _: () = { let
link_section = ".CRT$XCU"
 # ; };

#[cfg(all(target_vendor = "pc", not(any(target_env = "gnu", target_env = "msvc"))))]
 # const _: () = { let
link_section = ".ctors"
 # ; };

#[cfg(all(target_os = "aix"))]
 # const _: () = { let
link_section = ()
 # ; };

 // default
link_section = (compile_error! ("Unsupported target for #[ctor]"))
 # }
 ```

## `no_fail_on_missing_unsafe`

 ```rust
 # #[cfg(false)] {
#[cfg(no_fail_on_missing_unsafe)]
 # const _: () = { let
no_fail_on_missing_unsafe = (no_fail_on_missing_unsafe)
 # ; };

 // default
no_fail_on_missing_unsafe = ()
 # }
 ```

## `priority`

 ```rust
 # #[cfg(false)] {
#[cfg(feature = "priority")]
 # const _: () = { let
priority = early
 # ; };

 // default
priority = ()
 # }
 ```
