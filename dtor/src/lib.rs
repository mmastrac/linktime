#![recursion_limit = "256"]
#![no_std]
#![doc = include_str!("../docs/BUILD.md")]
//! # dtor
#![doc = include_str!("../docs/PREAMBLE.md")]
#![doc = include_str!("../docs/GENERATED.md")]
#![cfg_attr(linktime_used_linker, doc(test(attr(feature(used_with_arg)))))]

#[cfg(feature = "std")]
extern crate std;

mod macros;
mod native;
mod parse;

#[doc = include_str!("../docs/LIFE_BEFORE_MAIN.md")]
pub mod life_before_main {}

pub use native::*;

/// Marks a function as a library/executable destructor. This uses OS-specific
/// linker sections to call a specific function at termination time.
///
/// Multiple shutdown functions are supported, but the invocation order is not
/// guaranteed.
///
/// ```rust,ignore
/// # use dtor::dtor;
/// # fn main() {}
///
/// #[dtor(unsafe)]
/// fn shutdown() {
///   /* ... */
/// }
/// ```
#[doc(inline)]
#[cfg(feature = "proc_macro")]
pub use linktime_proc_macro::dtor;

/// Declarative forms of the `#[dtor]` macro.
///
/// The declarative forms wrap and parse a proc_macro-like syntax like so, and
/// are identical in expansion to the undecorated procedural macros. The
/// declarative forms support the same attribute parameters as the procedural
/// macros.
///
/// ```rust
/// # #[cfg(any())] mod test { use dtor::*; use libc_print::*;
/// dtor::declarative::dtor! {
///   #[dtor(unsafe)]
///   fn foo() {
///     libc_println!("Goodbye, world!");
///   }
/// }
/// # }
///
/// // ... the above is identical to:
///
/// # #[cfg(any())] mod test_2 { use dtor::*; use libc_print::*;
/// #[dtor(unsafe)]
/// fn foo() {
///   libc_println!("Goodbye, world!");
/// }
/// # }
/// ```
pub mod declarative {
    #[doc(inline)]
    pub use crate::__dtor_parse as dtor;
}

#[doc(hidden)]
#[allow(unused)]
pub mod __support {
    use crate::macros::*;

    // Required for proc_macro.
    pub use crate::__dtor_parse as dtor_parse;

    pub use crate::native::*;
}

__declare_features!(
    dtor: __dtor_features;

    /// Do not give the destructor's registration entry a name in the generated
    /// code (allows for multiple items with the same name). Equivalent to
    /// wrapping the registration in an anonymous const (i.e.: `const _ = { ... };`).
    anonymous {
        attr: [(anonymous) => (anonymous)];
    };
    /// Specify a custom crate path for the `dtor` crate. Used when re-exporting the dtor macro.
    crate_path {
        attr: [(crate_path = $path:pat) => (($path))];
        example: "crate_path = ::path::to::dtor::crate";
    };
    /// Specify a custom export name prefix for the generated constructor
    /// function.
    ///
    /// If specified, an export with the given prefix will be generated in the
    /// form:
    ///
    /// `<prefix>_<unique_id>`
    ctor_export_name_prefix {
        attr: [(ctor(export_name_prefix = $ctor_export_name_prefix_str:literal)) => ($ctor_export_name_prefix_str)];
        example: "ctor(export_name_prefix = \"ctor_\")";
        default {
            (target_os = "aix") => "__sinit80000000",
            _ => (),
        }
    };
    /// Place the generated registration constructor's function pointer in a
    /// custom link section.
    ctor_link_section {
        attr: [(ctor(link_section = $ctor_link_section_name:literal)) => ($ctor_link_section_name)];
        example: "ctor(link_section = \".ctors\")";
        default {
            // This is no longer supported by Apple
            (target_vendor = "apple") => "__DATA,__mod_init_func,mod_init_funcs",
            // Most LLVM/GCC targets can use .fini_array
            (any(
                target_os = "linux",
                target_os = "android",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
                target_os = "dragonfly",
                target_os = "illumos",
                target_os = "haiku",
                target_os = "vxworks",
                target_os = "nto",
                target_family = "wasm"
            )) => ".init_array",
            // No OS
            (target_os = "none") => ".init_array",
            // xtensa targets: .dtors
            (target_arch = "xtensa") => ".ctors",
            // Windows targets: .CRT$XCU
            (all(target_os = "windows", any(target_env = "gnu", target_env = "msvc"))) => ".CRT$XCU",
            // ... except GNU
            (all(target_os = "windows", not(any(target_env = "gnu", target_env = "msvc")))) => ".ctors",
            (all(target_os = "aix")) => (), // AIX uses export_name_prefix
            _ => (compile_error!("Unsupported target for #[ctor]"))
        }
    };
    /// The default method used for running a `dtor` on termination. This is
    /// generally not recommended as code may be unloaded before the dtor is
    /// called.
    ///
    /// This is only used if the specified `dtor` method is `term`.
    ///
    /// All platforms use `at_binary_exit` except Windows, which uses
    /// `at_module_exit`.
    default_term_method {
        default {
            (target_os = "windows") => at_module_exit,
            _ => at_binary_exit,
        }
    };
    /// The default method used for running a `dtor` on module unload.
    ///
    /// This is only used if the `method` attribute is not specified, or if the
    /// method is `unload`.
    default_unload_method {
        default {
            _ => at_module_exit,
        }
    };
    /// Specify a custom export name prefix for the destructor function.
    ///
    /// If specified, an export with the given prefix will be generated in the form:
    ///
    /// `<prefix>_<unique_id>`
    export_name_prefix {
        attr: [(export_name_prefix = $export_name_prefix_str:literal) => ($export_name_prefix_str)];
        example: "export_name_prefix = \"ctor_\"";
        default {
            (target_os = "aix") => "__sterm80000000",
            _ => (),
        }
    };
    /// Place the destructor function pointer in a custom link section.
    link_section {
        attr: [(link_section = $section:literal) => ($section)];
        example: "link_section = \".dtors\"";
        default {
            // This is no longer supported by Apple
            (target_vendor = "apple") => "__DATA,__mod_term_func,mod_term_funcs",
            // Most LLVM/GCC targets can use .fini_array
            (any(
                target_os = "linux",
                target_os = "android",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "dragonfly",
                target_os = "illumos",
                target_os = "haiku",
                target_os = "vxworks",
                target_os = "nto",
                target_family = "wasm"
            )) => ".fini_array",
            // OpenBSD only seems to support .fini_array for binaries
            (target_os = "openbsd") => ".dtors",
            // No OS
            (target_os = "none") => ".fini_array",
            // xtensa targets: .dtors
            (target_arch = "xtensa") => ".dtors",
            // Windows targets: .CRT$XPU (requires static CRT)
            (all(target_os = "windows", any(target_env = "gnu", target_env = "msvc"))) => ".CRT$XPU",
            // ... except GNU
            (all(target_os = "windows", not(any(target_env = "gnu", target_env = "msvc")))) => ".dtors",
            (all(target_os = "aix")) => (), // AIX uses export_name_prefix
            _ => (compile_error!("Unsupported target for #[dtor]"))
        }
    };
    /// Specify the dtor method.
    ///
    ///  - `term`: Run the dtor on binary termination using the platform's
    ///    [default_term_method](#default_term_method). Not recommended as code
    ///    may be unloaded before the dtor is called.
    ///  - `unload`: Run the dtor on module unload (library or binary) using the
    ///    platform's [default_unload_method](#default_unload_method).
    ///  - `at_module_exit`: Run the dtor using the platform's
    ///    [`at_module_exit`][at_module_exit] (`__cxa_atexit` on all platforms
    ///    other than Windows, `atexit` on Windows).
    ///  - `at_binary_exit`: Run the dtor using the platform's
    ///    [`at_binary_exit`][at_binary_exit] (unsupported on Windows
    ///    platforms).
    ///  - `linker`: Register the dtor using the platform's
    ///    [link_section](#link_section) or
    ///    [export_name_prefix](#export_name_prefix) (unsupported on Apple
    ///    platforms).
    ///
    /// [at_module_exit]: crate::native::at_module_exit
    /// [at_binary_exit]: crate::native::at_binary_exit
    method {
        attr: [(method = $method_id:ident) => ($method_id)];
        example: "method = term|unload|at_module_exit|at_binary_exit|linker";
        validate: [(term), (unload), (at_module_exit), (at_binary_exit), (linker)];
        default {
            (target_vendor = "apple") => at_module_exit,
            (target_os = "windows") => at_module_exit,
            // WASI/Emscripten support atexit only
            // For wasm-unknown-unknown, you'll need to provide one
            (target_family = "wasm") => at_binary_exit,
            // OpenBSD's linker support is inconsistent
            (target_os = "openbsd") => at_module_exit,
            _ => linker,
        }
    };
    /// Enable support for the proc-macro `#[dtor]` attribute. The declarative
    /// form (`dtor!(...)`) is always available. It is recommended that crates
    /// re-exporting the `dtor` macro disable this feature and only use the
    /// declarative form.
    proc_macro {
        feature: "proc_macro";
    };
    /// Enable support for the standard library.
    std {
        feature: "std";
    };
    r#unsafe {
        /// attr
        ///
        /// Marks a dtor as unsafe. Required.
        ///
        /// The `dtor` crate rejects `#[dtor]` without marking the item unsafe;
        /// that error can be suppressed by passing
        /// `RUSTFLAGS="--cfg linktime_no_fail_on_missing_unsafe"` to Cargo.
        attr: [(unsafe) => (no_fail_on_missing_unsafe)];
        default {
            (linktime_no_fail_on_missing_unsafe) => (no_fail_on_missing_unsafe),
            _ => (),
        }
    };
    used_linker {
        /// attr
        ///
        /// Mark generated function pointers `used(linker)`. Requires nightly
        /// for the nightly-only feature `feature(used_with_arg)` (see
        /// <https://github.com/rust-lang/rust/issues/93798>).
        ///
        /// This can be made the default by using the `cfg` flag
        /// `linktime_used_linker` (`RUSTFLAGS="--cfg linktime_used_linker"`).
        ///
        /// For a crate using this macro to function correctly with and without
        /// this flag, it is recommended to add the following line to the top of
        /// lib.rs in the crate root:
        ///
        /// `#![cfg_attr(linktime_used_linker, feature(used_with_arg))]`
        ///
        attr: [(used(linker)) => (used_linker)];
        default {
            (linktime_used_linker) => used_linker,
            _ => (),
        }
    };
);

#[cfg(doc)]
__generate_docs!(__dtor_features);
