//! macro-magic integration tests.
#![allow(unexpected_cfgs)]
#![recursion_limit = "256"]

use ::macro_magic::*;

__test!(__parse_feature_input[$]:
    (
        my_macro: my_macro_parse;

        link_section {
            attr: [(link_section($section:literal)) => ($section)];
            default {
                (target_vendor = "apple") => "__DATA,__mod_term_func,mod_term_funcs",
                (target_os = "windows") => ".CRT$SOMETHING",
                (target_os = "linux") => ".dtors",
                // Fallback for tier-3.
                _ => ".dtors"
            }
        };
    ) => (
        (my_macro my_macro_parse)
        ((
            feature = link_section;
            docs = [];
            attr = [(link_section($section:literal)) => ($section)];
            attr_docs = [];
            example = ((stringify! (link_section($section:literal))));
            validate = ([$link_section : tt]);
            default = [
                ((target_vendor = "apple") => "__DATA,__mod_term_func,mod_term_funcs")
                ((target_os = "windows") => ".CRT$SOMETHING")
                ((target_os = "linux") => ".dtors")
                (_ => ".dtors")
                (_ => ())
            ]
        ))
    )
);

__test!(__process_defaults:
    (
        (
            feature = link_section;
            attr = [(link_section($section:literal)) => ($section)];
            attr_docs = [];
            example = ();
            validate = [];
            default = [
                ((a = "apple") => 1)
                ((b = "pc") => (compile_error!("2")))
                ((c = "linux") => 3)
                (_ => (compile_error!("4")))
            ]
        )
    ) => (
        (
            feature = link_section;
            feature_attr = link_section;
            attr = [(link_section($section:literal)) => ($section)];
            attr_docs = [];
            example = ();
            validate = [];
            original_defaults = {((a = "apple") => 1)((b = "pc") => (compile_error!("2")))((c = "linux") => 3)(_ => (compile_error!("4")))};
            default = [
                ((all(a = "apple", not(any()))) => 1)
                ((all(b = "pc", not(any(a = "apple",)))) => (compile_error!("2")))
                ((all(c = "linux", not(any(a = "apple", b = "pc",)))) => 3)
                ((not(any(a = "apple", b = "pc", c = "linux",))) => (compile_error!("4")))
            ]
        )
    )
);

__declare_features!(
    small_macro: small_macro_parse;

    /// Enable support for the standard library. This is required for static
    /// ctor variables, but not for functions.
    std {
        feature: "std";
    };
    priority {
        attr: [(priority = $priority_value:tt) => ($priority_value)];
    };
    unsafe {
        attr: [(unsafe) => (unsafe)];
    };
);

__declare_features!(
    my_macro: my_macro_parse;

    /// Enable support for the standard library. This is required for static
    /// ctor variables, but not for functions.
    std {
        feature: "std";
    };
    /// Mark all ctor functions with `used(linker)`.
    used_linker {
        feature: "used_linker";
        attr: [(used(linker)) => (used_linker)];
    };
    /// Enable support for the proc-macro `#[ctor]` and `#[dtor]` attributes.
    proc_macro {
        feature: "proc_macro";
    };
    /// Do not warn when a ctor or dtor is missing the `unsafe` keyword.
    no_warn_on_missing_unsafe {
        feature: "no_warn_on_missing_unsafe";
        attr: [(unsafe) => (no_warn_on_missing_unsafe)];
    };
    /// Enable support for the priority parameter.
    priority_enabled {
        feature: "priority";
    };
    /// Set the ctor priority to a given value.
    priority {
        attr: [(priority = $priority_value:tt) => ($priority_value)];
        validate: [($priority:literal), (early), (late)];
    };
    /// Place the initialization function pointer in a custom link section. This
    /// may cause the initialization function to fail to run or run earlier or
    /// later than other `ctor` functions.
    link_section {
        attr: [(link_section($section:literal)) => ($section)];
        default {
            (target_vendor = "apple") => "__DATA,__mod_term_func,mod_term_funcs",
            (target_os = "windows") => ".fini_array",
            (target_os = "linux") => ".dtors",
            (target_os = "freebsd") => ".fini",
            _ => ".dtors"
        }
    };
    /// Specify a custom crate path for the `ctor` crate. Used when re-exporting the ctor macro.
    crate_path {
        attr: [(crate_path = $path:pat) => ($path)];
    };
    /// Make the ctor function anonymous.
    anonymous {
        attr: [(anonymous) => (anonymous)];
    };
);

__test!(my_macro_parse[my_macro_parse => @extract (std)]:
    (std = std_value,) => (std = std_value,));
__test!(my_macro_parse[my_macro_parse => @extract (std)]:
    (std = std_value, anonymous = (),) => (std = std_value,));
__test!(my_macro_parse[my_macro_parse => @extract (std)]:
    (std = std_value, std = second_value,) => (std = second_value,));
__test!(my_macro_parse[my_macro_parse => @extract (std anonymous)]:
    (std = std_value, anonymous = (),) => (std = std_value, anonymous = (),));
__test!(my_macro_parse[my_macro_parse => @extract (std anonymous)]:
    (std = std_value,) => (std = std_value, anonymous = (),));

__test!(__extract_meta[small_macro_parse]:
    (small_macro) => (std = std default, priority = ()default, unsafe = ()default,));
__test!(__extract_meta[small_macro_parse]:
    (small_macro()) => (std = std default, priority = ()default, unsafe = ()default,));
__test!(__extract_meta[small_macro_parse]:
    (small_macro(unsafe)) => (std = std default, priority = ()default, unsafe = unsafe value,));
__test!(__extract_meta[small_macro_parse]:
    (small_macro(unsafe, priority = 1)) => (std = std default , priority = 1 value , unsafe = unsafe value ,));
__test!(__extract_meta[small_macro_parse]:
    (small_macro(priority = 1)) => (std = std default , priority = 1 value , unsafe = () default ,));

#[cfg(target_vendor = "apple")]
__test!(my_macro_parse[my_macro_parse => @crate]:
    () => (
        std = std, used_linker = (), proc_macro = (), no_warn_on_missing_unsafe = (),
        priority_enabled = (), priority = (), 
        link_section = "__DATA,__mod_term_func,mod_term_funcs", 
        crate_path = (), anonymous = (),));

#[cfg(target_vendor = "apple")]
__test!(__extract_meta[my_macro_parse]:
    (my_macro(used(linker))) => (
        std = std default, used_linker = used_linker value, proc_macro = ()default,
        no_warn_on_missing_unsafe = ()default, priority_enabled = ()default, priority
        = ()default, link_section = "__DATA,__mod_term_func,mod_term_funcs" default,
        crate_path = ()default, anonymous = ()default,));

__test!(my_macro_parse[my_macro_parse => @self]:
    (#[my_macro]) => ((my_macro)()));
__test!(my_macro_parse[my_macro_parse => @self]:
    (#[my_macro(unsafe)]) => ((my_macro(unsafe))()));
__test!(my_macro_parse[my_macro_parse => @self]:
    (#[my_macro(unsafe, priority = 1)]) => ((my_macro(unsafe, priority = 1))()));
__test!(my_macro_parse[my_macro_parse => @self]:
    (#[my_macro(unsafe, priority = 1)] #[other_macro]) => ((my_macro(unsafe, priority = 1))(#[other_macro])));
