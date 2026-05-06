#![doc = include_str!("../docs/BUILD.md")]
//! # link-section
#![doc = include_str!("../docs/PREAMBLE.md")]
#![allow(unsafe_code)]
#![cfg_attr(linktime_used_linker, doc(test(attr(feature(used_with_arg)))))]
#![no_std]

#[doc = include_str!("../docs/LIFE_BEFORE_MAIN.md")]
pub mod life_before_main {}

#[cfg(target_family = "wasm")]
mod wasm;

#[cfg(target_family = "wasm")]
extern crate alloc;

/// Declarative forms of the `#[section]` and `#[in_section(...)]` macros.
///
/// The declarative forms wrap and parse a proc_macro-like syntax like so, and
/// are identical in expansion to the undecorated procedural macros. The
/// declarative forms support the same attribute parameters as the procedural
/// macros.
pub mod declarative {
    pub use crate::__in_section_parse as in_section;
    pub use crate::__section_parse as section;
}

#[doc(hidden)]
pub mod __support {
    pub use crate::__add_section_link_attribute as add_section_link_attribute;
    pub use crate::__def_section_name as def_section_name;
    pub use crate::__get_section as get_section;
    pub use crate::__in_section_crate as in_section_crate;
    pub use crate::__in_section_parse as in_section_parse;
    pub use crate::__section_parse as section_parse;

    #[cfg(feature = "proc_macro")]
    pub use linktime_proc_macro::hash;
    #[cfg(feature = "proc_macro")]
    pub use linktime_proc_macro::ident_concat;

    #[cfg(target_family = "wasm")]
    extern "C" {
        /// Read custom section with name/name_length as a UTF8 string
        pub(crate) fn read_custom_section(
            name: *const u8,
            name_length: usize,
            target_address: *mut u8,
            target_address_length: usize,
        ) -> usize;
    }

    #[cfg(target_family = "wasm")]
    pub use crate::wasm::{register_wasm_link_section_item, LinkSectionRawInfo};

    #[cfg(all(not(miri), target_vendor = "pc"))]
    pub use section::Alignment;

    /// Declares the section_name macro.
    #[macro_export]
    #[doc(hidden)]
    macro_rules! __def_section_name {
        (
            {$(
                $__section:ident $__type:ident => $__prefix:tt __ $__suffix:tt;
            )*}
            AUXILIARY = $__aux_sep:literal;
            MAX_LENGTH = $__max_length:literal;
            HASH_LENGTH = $__hash_length:literal;
            VALID_SECTION_CHARS = $__valid_section_chars:literal;
        ) => {
            /// Internal macro for generating a section name.
            #[macro_export]
            #[doc(hidden)]
            macro_rules! __section_name {
                $(
                    (raw $__section $__type $name:ident) => {
                        concat!(concat! $__prefix, stringify!($name), concat! $__suffix);
                    };
                    (raw $__section $__type $name:ident $aux:ident) => {
                        concat!(concat! $__prefix, stringify!($name), $__aux_sep, stringify!($aux), concat! $__suffix);
                    };
                    ($pattern:tt $__section $__type $name:ident) => {
                        $crate::__support::hash!($pattern ($__prefix) $name ($__suffix) $__hash_length $__max_length $__valid_section_chars);
                    };
                    ($pattern:tt $__section $__type $name:ident $aux:ident) => {
                        $crate::__support::hash!($pattern ($__prefix) ($name $__aux_sep $aux) ($__suffix) $__hash_length $__max_length $__valid_section_chars);
                    };
                )*
                ($pattern:tt $unknown_section:ident $unknown_type:ident $name:ident) => {
                    const _: () = {
                        compile_error!(concat!("Unknown section type: `", stringify!($unknown_section), "/", stringify!($unknown_type), "`"));
                    };
                };
            }
        };
    }

    #[cfg(not(linktime_used_linker))]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __add_used {
        (
            $section:ident $type:ident $name:ident $($aux:ident)? #[$attr:ident = __]
            $(#[$meta:meta])*
            $vis:vis static $($static:tt)*
        ) => {
            $crate::__add_section_link_attribute_impl!(
                $section $type $name $($aux)? #[$attr = __]
                $(#[$meta])*
                #[used]
                $vis static $($static)*
            );
        };
    }

    #[cfg(linktime_used_linker)]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __add_used {
        (
            $section:ident $type:ident $name:ident $($aux:ident)? #[$attr:ident = __]
            $(#[$meta:meta])*
            $vis:vis static $($static:tt)*
        ) => {
            $crate::__add_section_link_attribute_impl!(
                $section $type $name $($aux)? #[$attr = __]
                $(#[$meta])*
                #[used(linker)]
                $vis static $($static)*
            );
        };
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! __add_section_link_attribute(
        ($section:ident $type:ident $name:ident $($aux:ident)? #[$attr:ident = __]
            $(#[$meta:meta])*
            $vis:vis static $($static:tt)*
        ) => {
            $crate::__add_used!(
                $section $type $name $($aux)? #[$attr = __]
                $(#[$meta])*
                $vis static $($static)*
            );
        };
        ($section:ident $type:ident $name:ident $($aux:ident)? #[$attr:ident = __]
            extern "C" {
                $(#[$meta:meta])*
                $vis:vis static $($static:tt)*
            }
        ) => {
            extern "C" {
                $crate::__add_section_link_attribute_impl!(
                    $section $type $name $($aux)? #[$attr = __]
                    $(#[$meta])*
                    #[allow(unsafe_code)]
                    $vis static $($static)*
                );
            }
        };
        ($section:ident $type:ident $name:ident $($aux:ident)? #[$attr:ident = __]
            $($item:tt)*) => {
            $crate::__add_section_link_attribute_impl!(
                $section $type $name $($aux)? #[$attr = __]
                #[allow(unsafe_code)]
                $($item)*
            );
        };
    );

    #[cfg(feature = "proc_macro")]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __add_section_link_attribute_impl(
        ($section:ident $type:ident $name:ident $($aux:ident)? #[$attr:ident = __] $($item:tt)*) => {
            $crate::__section_name!(
                (#[$attr = __] #[allow(unsafe_code)] $($item)*)
                $section $type $name $($aux)?
            );
        }
    );

    #[cfg(not(feature = "proc_macro"))]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __add_section_link_attribute_impl(
        ($section:ident $type:ident $name:ident #[$attr:ident = __] $($item:tt)*) => {
            #[$attr = $crate::__section_name!(
                raw $section $type $name
            )] $($item)*
        }
    );

    // \x01: "do not mangle" (ref https://github.com/rust-lang/rust-bindgen/issues/2935)
    #[cfg(target_vendor = "apple")]
    def_section_name! {
        {
            data bare =>    ("__DATA,") __ ();
            code bare =>    ("__TEXT,") __ ();
            data section => ("__DATA,") __ (",regular,no_dead_strip");
            code section => ("__TEXT,") __ (",regular,pure_instructions");
            data start =>   ("\x01section$start$__DATA$") __ ();
            data end =>     ("\x01section$end$__DATA$") __ ();
        }
        AUXILIARY = "_";
        MAX_LENGTH = 16;
        HASH_LENGTH = 6;
        VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    }

    #[cfg(target_family = "wasm")]
    def_section_name! {
        {
            data bare =>    (".data", ".link_section.") __ ();
            data section => (".data", ".link_section.") __ ();
            code bare =>    (".text", ".link_section.") __ ();
            code section => (".text", ".link_section.") __ ();
            data bounds =>   (".data", ".link_section.") __ (".bounds");
        }
        AUXILIARY = ".";
        MAX_LENGTH = 16;
        HASH_LENGTH = 6;
        VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    }

    #[cfg(all(
        not(target_vendor = "apple"),
        not(target_vendor = "pc"),
        not(target_family = "wasm")
    ))]
    def_section_name! {
        {
            data bare =>    ("_data", "_link_section_") __ ();
            data section => ("_data", "_link_section_") __ ();
            data start =>   ("__start_", "_data", "_link_section_") __ ();
            data end =>     ("__stop_", "_data", "_link_section_") __ ();
            code bare =>    ("_text", "_link_section_") __ ();
            code section => ("_text", "_link_section_") __ ();
            code start =>   ("__start_", "_text", "_link_section_") __ ();
            code end =>     ("__stop_", "_text", "_link_section_") __ ();
        }
        AUXILIARY = "_";
        MAX_LENGTH = 64;
        HASH_LENGTH = 10;
        VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    }

    #[cfg(target_vendor = "pc")]
    def_section_name! {
        {
            data bare =>    (".data", "$") __ ();
            data section => (".data", "$") __ ("$b");
            data start =>   (".data", "$") __ ("$a");
            data end =>     (".data", "$") __ ("$c");
            code bare =>    (".text", "$") __ ();
            code section => (".text", "$") __ ("$b");
            code start =>   (".text", "$") __ ("$a");
            code end =>     (".text", "$") __ ("$c");
        }
        AUXILIARY = "$d$";
        MAX_LENGTH = 64;
        HASH_LENGTH = 10;
        VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    }

    #[cfg(not(feature = "proc_macro"))]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __declare_macro {
        ($vis:vis $ident:ident $generic_macro:ident $args:tt) => {
            /// Internal macro for parsing the section. This is exported with
            /// the same name as the type below.
            #[doc(hidden)]
            $vis use $crate::$generic_macro as $ident;
        };
    }

    #[cfg(feature = "proc_macro")]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __declare_macro {
        ($vis:vis $ident:ident $generic_macro:ident $args:tt) => {
            $crate::__support::ident_concat!(
                (#[macro_export]
                #[doc(hidden)]
                macro_rules!)  (__ $ident __link_section_private_macro__) ({
                    ($passthru:tt) => {
                        $crate::$generic_macro!($passthru $args);
                    };
                })
            );

            $crate::__support::ident_concat!(
                (#[doc(hidden)] pub use) (__ $ident __link_section_private_macro__) (as $ident;)
            );
        }
    }

    /// Define a link section.
    #[macro_export]
    #[doc(hidden)]
    macro_rules! __section_parse {
        // Has a generic (note that $generic eats the trailing semicolon)
        (#[section $($args:tt)*] $(#[$meta:meta])* $vis:vis static $ident:ident : $(:: $path_prefix:ident ::)? $($path:ident)::* < $($generic:tt)*) => {
            $crate::__section_parse!(@parsed #[section $($args)*] $(#[$meta])* $vis static $ident: ( $(:: $path_prefix ::)? $($path)::* < $($generic)*) ( $($generic)* ) __in_section_helper_macro_generic);
        };
        // No generic
        (#[section $($args:tt)*] $(#[$meta:meta])* $vis:vis static $ident:ident : $(:: $path_prefix:ident ::)? $($path:ident)::* ;) => {
            $crate::__section_parse!(@parsed #[section $($args)*] $(#[$meta])* $vis static $ident: ( $(:: $path_prefix ::)? $($path)::* ;) ( () > ; ) __in_section_helper_macro_no_generic);
        };
        // Both end up here...
        (@parsed #[section(aux = $name:ident, no_macro)] $(#[$meta:meta])* $vis:vis static $ident:ident : ($ty:ty ;) ( $generic_ty:ty > ; ) $generic_macro:ident) => {
            $crate::__section_parse!(@generate #[section(aux = $name)] $(#[$meta])* $vis static $ident: $ty, $generic_ty, $generic_macro);
        };
        (@parsed #[section(no_macro)] $(#[$meta:meta])* $vis:vis static $ident:ident : ($ty:ty ;) ( $generic_ty:ty > ; ) $generic_macro:ident) => {
            $crate::__section_parse!(@generate #[section] $(#[$meta])* $vis static $ident: $ty, $generic_ty, $generic_macro);
        };
        (@parsed #[section $($args:tt)*] $(#[$meta:meta])* $vis:vis static $ident:ident : ($ty:ty ;) ( $generic_ty:ty > ; ) $generic_macro:ident) => {
            $crate::__declare_macro!($vis $ident $generic_macro ($($args)*));
            $crate::__section_parse!(@generate #[section $($args)*] $(#[$meta])* $vis static $ident: $ty, $generic_ty, $generic_macro);
        };
        (@generate #[section $($args:tt)*] $(#[$meta:meta])* $vis:vis static $ident:ident : $ty:ty, $generic_ty:ty, __in_section_helper_macro_generic) => {
            $crate::__section_parse!(@generate #[section $($args)*] $(#[$meta])* $vis static $ident: $ty, $generic_ty, __base_case__);

            impl ::core::iter::IntoIterator for $ident {
                type Item = &'static $generic_ty;
                type IntoIter = ::core::slice::Iter<'static, $generic_ty>;
                fn into_iter(self) -> Self::IntoIter {
                    $ident.as_slice().iter()
                }
            }
        };
        (@generate #[section $($args:tt)*] $(#[$meta:meta])* $vis:vis static $ident:ident : $ty:ty, $generic_ty:ty, $generic_macro:ident) => {
            $(#[$meta])*
            #[allow(non_camel_case_types)]
            $vis struct $ident;

            impl $crate::__support::SectionItemType for $ident {
                type Item = $generic_ty;
            }

            impl ::core::fmt::Debug for $ident {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    ::core::ops::Deref::deref(self).fmt(f)
                }
            }

            impl ::core::ops::Deref for $ident {
                type Target = $ty;
                fn deref(&self) -> &Self::Target {
                    self.const_deref()
                }
            }

            $crate::__section_parse!(@deref #[section $($args)*] $(#[$meta])* $vis static $ident: $ty, $generic_ty, __base_case__);
        };
        (@deref #[section] $(#[$meta:meta])* $vis:vis static $ident:ident : $ty:ty, $generic_ty:ty, __base_case__) => {
            impl $ident {
                /// Get a `const` reference to the underlying section. In
                /// non-const contexts, `deref` is sufficient.
                pub const fn const_deref(&self) -> &$ty {
                    static SECTION: $ty = {
                        let section = $crate::__support::get_section!(name=$ident, type=$generic_ty, aux=);
                        let name = $crate::__section_name!(
                            raw data bare $ident
                        );
                        unsafe { <$ty>::new(name, section) }
                    };
                    &SECTION
                }
            }
        };
        (@deref #[section(aux=$aux:ident)] $(#[$meta:meta])* $vis:vis static $ident:ident : $ty:ty, $generic_ty:ty, __base_case__) => {
            impl $ident {
                /// Get a `const` reference to the underlying section. In
                /// non-const contexts, `deref` is sufficient.
                pub const fn const_deref(&self) -> &$ty {
                    static SECTION: $ty = {
                        let section = $crate::__support::get_section!(name=$ident, type=$generic_ty, aux=$aux);
                        let name = $crate::__section_name!(
                            raw data bare $aux $ident // swap
                        );
                        unsafe { <$ty>::new(name, section) }
                    };
                    &SECTION
                }
            }
        };
    }

    #[cfg(miri)]
    mod section {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! __get_section {
            (name=$ident:ident, type=$generic_ty:ty, aux=$($aux:ident)?) => {{
                $crate::__support::PtrBounds::new(core::ptr::null_mut(), core::ptr::null_mut())
            }};
        }

        pub type Bounds = crate::__support::PtrBounds;
    }

    #[cfg(all(not(miri), target_family = "wasm"))]
    mod section {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! __get_section {
            (name=$ident:ident, type=$generic_ty:ty, aux=$($aux:ident)?) => {
                {
                    static __LINK_SECTION_NAME: &'static str = $crate::__section_name!(
                        raw data bare $ident $($aux)?
                    );
                    $crate::__support::add_section_link_attribute!(
                        data bounds $ident $($aux)?
                        #[export_name = __]
                        #[used]
                        static mut __LINK_SECTION_INFO: $crate::__support::LinkSectionRawInfo = $crate::__support::LinkSectionRawInfo::new::<$generic_ty>(__LINK_SECTION_NAME);
                    );

                    unsafe { $crate::__support::Bounds::new(&raw mut __LINK_SECTION_INFO) }
                }
            }
        }

        pub use crate::wasm::Bounds;
    }

    /// On Windows platforms we don't have start/end symbols, but we do have
    /// section sorting so we drop a minimum-sized type with the same alignment
    /// as T at the start and end of the section.
    #[cfg(all(not(miri), target_vendor = "pc"))]
    mod section {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! __get_section {
            (name=$ident:ident, type=$generic_ty:ty, aux=$($aux:ident)?) => {
                {
                    use $crate::__support::Alignment;
                    use $crate::__support::PtrBounds;
                    use $crate::__support::add_section_link_attribute;
                    use core::mem;

                    add_section_link_attribute!(
                        data start $ident $($aux)?
                        #[link_section = __]
                        static __START: Alignment<$generic_ty> = Alignment::new();
                    );
                    add_section_link_attribute!(
                        data end $ident $($aux)?
                        #[link_section = __]
                        static __END: Alignment<$generic_ty> = Alignment::new();
                    );

                    PtrBounds::new(
                        unsafe {
                            let start = &raw const __START;
                            start.cast::<u8>().add(mem::size_of::<Alignment<$generic_ty>>()) as *const()
                        },
                        unsafe { &raw const __END as *const () },
                    )
                }
            }
        }

        /// A non-zero-sized type that is used to align the start and end of the
        /// section.
        #[repr(C)]
        pub struct Alignment<T> {
            _align: [T; 0],
            _padding: u8,
        }

        #[allow(clippy::new_without_default)]
        impl<T> Alignment<T> {
            pub const fn new() -> Self {
                Self {
                    _align: [],
                    _padding: 0,
                }
            }
        }

        pub type Bounds = crate::__support::PtrBounds;
    }

    /// On LLVM/GCC platforms we can use orphan sections with _start and _end
    /// symbols.
    ///
    /// On Apple platforms, the linker provides a pointer to the start and end
    /// of the section regardless of the section's name.
    #[cfg(all(not(miri), not(target_family = "wasm"), not(target_vendor = "pc")))]
    mod section {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! __get_section {
            (name=$ident:ident, type=$generic_ty:ty, aux=$($aux:ident)?) => {
                {
                    // These are not valid items, but they are valid pointers.
                    // We cannot safely use them - only take pointers to them.
                    $crate::__support::add_section_link_attribute!(
                        data start $ident $($aux)?
                        #[link_name = __]
                        extern "C" {
                            static __START: u8;
                        }
                    );
                    $crate::__support::add_section_link_attribute!(
                        data end $ident $($aux)?
                        #[link_name = __]
                        extern "C" {
                            static __END: u8;
                        }
                    );

                    $crate::__support::PtrBounds::new(
                        // TODO: black_box when hint is stable
                        unsafe { &raw const __START as *const () },
                        unsafe { &raw const __END as *const () },
                    )
                }
            }
        }

        pub type Bounds = crate::__support::PtrBounds;
    }

    /// Export a symbol into a link section.
    #[macro_export]
    #[doc(hidden)]
    macro_rules! __in_section_parse {
        // Needs to handle:
        //  <name>
        //  :: <name>
        //  <path> :: <name>
        //  :: <path> :: <name>
        //  etc...
        (#[in_section(unsafe, type = $stored_ty:ty, name = $ident:ident $( , aux = $aux:ident)?)] $($item:tt)*) => {
            $crate::__support::in_section_crate!((type = $stored_ty), $ident, $($aux)?, $ident, ($($item)*));
        };
        (#[in_section(unsafe, name = $ident:ident $( , aux = $aux:ident)?)] $($item:tt)*) => {
            $crate::__support::in_section_crate!(data, $ident, $($aux)?, $ident, ($($item)*));
        };
        (#[in_section( $($path:tt)* )] $($item:tt)*) => {
            $crate::__support::in_section_parse!(path=[$($path)*] #[in_section($($path)*)] $($item)*);
        };
        (path=[$orig_path:path] #[in_section($name:ident)] $($item:tt)*) => {
            $orig_path ! (
                (v=0 (name=$name (path=[$orig_path] (item=($($item)*) ()))))
            );
        };
        (path=[$orig_path:path] #[in_section(:: $($path:ident)::*)] $($item:tt)*) => {
            $crate::__support::in_section_parse!(path=[$orig_path] #[in_section($($path)::*)] $($item)*);
        };
        (path=[$orig_path:path] #[in_section($prefix:ident :: $($path:ident)::*)] $($item:tt)*) => {
            $crate::__support::in_section_parse!(path=[$orig_path] #[in_section($($path)::*)] $($item)*);
        };
    }

    #[macro_export]
    #[doc(hidden)]
    macro_rules! __in_section_helper_macro_generic {
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt))))) => {
            $crate::__support::in_section_crate!(section, $ident,, $path, $item);
        };
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt)))) ((aux=$aux:ident)) )=> {
            $crate::__support::in_section_crate!(section, $ident, $aux, $path, $item);
        };
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt)))) () )=> {
            $crate::__support::in_section_crate!(section, $ident,, $path, $item);
        };
        ((v=$v:literal $rest:tt)) => {
            const _: () = {
                compile_error!(concat!(
                    "link-section: Unsupported version: `",
                    stringify!($v),
                    "`: ",
                    stringify!($rest)
                ));
            };
        };
    }

    #[macro_export]
    #[doc(hidden)]
    macro_rules! __in_section_helper_macro_no_generic {
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt))))) => {
            $crate::__support::in_section_crate!(data, $ident,, $path, $item);
        };
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt)))) ((aux=$aux:ident)) )=> {
            $crate::__support::in_section_crate!(data, $ident, $aux, $path, $item);
        };
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt)))) () )=> {
            $crate::__support::in_section_crate!(data, $ident,, $path, $item);
        };
        ((v=$v:literal $rest:tt)) => {
            const _: () = {
                compile_error!(concat!(
                    "link-section: Unsupported version: `",
                    stringify!($v),
                    "`: ",
                    stringify!($rest)
                ));
            };
        };
    }

    #[macro_export]
    #[doc(hidden)]
    #[allow(unknown_lints, edition_2024_expr_fragment_specifier)]
    macro_rules! __in_section_crate {
        (@type_select section $path:path, $item_ty:ty) => {
            <$path as $crate::__support::SectionItemType>::Item
        };
        (@type_select data $path:path, $item_ty:ty) => {
            $item_ty
        };
        (@type_select (type = $stored_ty:ty) $path:path, $item_ty:ty) => {
            $stored_ty
        };
        ($type_source:tt, $ident:ident, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis fn $ident_fn:ident($($args:tt)*) $(-> $ret:ty)? $body:block)) => {
            $crate::__in_section_crate!($type_source, $ident, $($aux)?, $path, (
                const _: fn($($args)*) $(-> $ret)? = $ident_fn;
            ));
            $crate::__add_section_link_attribute!(
                code section $ident $($aux)?
                #[link_section = __]
                $(#[$meta])*
                $vis fn $ident_fn($($args)*) $(-> $ret)? $body
            );
        };
        ($type_source:tt, $ident:ident, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis static _ : $ty:ty = $value:expr;)) => {
            $crate::__in_section_crate!($type_source, $ident, $($aux)?, $path, (
                $(#[$meta])*
                const _: fn($($args)*) $(-> $ret)? = $ident_fn;
            ));
        };
        ($type_source:tt, $ident:ident, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis static $ident_static:ident : $ty:ty = $value:expr;)) => {
            #[cfg(target_family = "wasm")]
            compile_error!("static items are not supported on WASM: use const items instead");

            #[cfg(not(target_family = "wasm"))]
            $crate::__add_section_link_attribute!(
                data section $ident $($aux)?
                #[link_section = __]
                $(#[$meta])*
                $vis static $ident_static: $crate::__in_section_crate!(@type_select $type_source $path, $ty) = $value;
            );
        };
        ($type_source:tt, $ident:ident, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis const $name:ident: $ty:ty = $value:expr;)) => {
            $(#[$meta])* $vis const $name: $ty = {
                type __InSecStoredTy = $crate::__in_section_crate!(@type_select $type_source $path, $ty);
                const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = $value;

                #[cfg(target_family = "wasm")]
                {
                    // Register a counting item
                    $crate::__add_section_link_attribute!(
                        data section $ident $($aux)?
                        #[link_section = __]
                        $vis static __LINK_SECTION_CONST_ITEM: u8 = 0;
                    );

                    $crate::__add_section_link_attribute!(
                        data bounds $ident $($aux)?
                        #[link_name = __]
                        extern "C" {
                            static mut __LINK_SECTION_INFO: $crate::__support::LinkSectionRawInfo;
                        }
                    );

                    #[link_section = ".init_array.0"]
                    static mut __LINK_SECTION_ITEM_FN_REF: extern "C" fn() = {
                        extern "C" fn __LINK_SECTION_ITEM_FN() {
                            static DISARMED: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);
                            if DISARMED.swap(true, ::core::sync::atomic::Ordering::Relaxed) {
                                return;
                            }
                            unsafe {
                                let ptr = $crate::__support::register_wasm_link_section_item(&raw mut __LINK_SECTION_INFO);
                                ::core::ptr::write(ptr as *mut __InSecStoredTy, __LINK_SECTION_CONST_ITEM_VALUE);
                            }
                        }
                        __LINK_SECTION_ITEM_FN
                    };
                }

                #[cfg(not(target_family = "wasm"))]
                $crate::__add_section_link_attribute!(
                    data section $ident $($aux)?
                    #[link_section = __]
                    $(#[$meta])*
                    $vis static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = __LINK_SECTION_CONST_ITEM_VALUE;
                );

                __LINK_SECTION_CONST_ITEM_VALUE
            };
        };
        // Simplify anonymous constants.
        ($type_source:tt, $ident:ident, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis const _: $ty:ty = $value:expr;)) => {
            $(#[$meta])* $vis const _: () = {
                type __InSecStoredTy = $crate::__in_section_crate!(@type_select $type_source $path, $ty);
                $crate::__add_section_link_attribute!(
                    data section $ident $($aux)?
                    #[link_section = __]
                    $(#[$meta])*
                    $vis static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = $value;
                );
            };
        };
        (data, $ident:ident, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $item:item)) => {
            $crate::__add_section_link_attribute!(
                data section $ident $($aux)?
                #[link_section = __]
                $(#[$meta])*
                $item
            );
        };
    }

    pub trait SectionItemType {
        type Item;
    }

    impl<T> SectionItemType for super::TypedSection<T> {
        type Item = T;
    }

    pub struct PtrBounds {
        pub start: *const (),
        pub end: *const (),
    }

    impl PtrBounds {
        pub const fn new(start: *const (), end: *const ()) -> Self {
            Self { start, end }
        }

        #[inline(always)]
        pub const fn start_ptr(&self) -> *const () {
            self.start
        }
        #[inline(always)]
        pub const fn end_ptr(&self) -> *const () {
            self.end
        }
        #[inline(always)]
        pub const fn byte_len(&self) -> usize {
            // NOTE: MSRV for non-WASM targets doesn't allow byte_offset_from,
            // so we manually implement it here.
            unsafe { (self.end.cast::<u8>()).offset_from(self.start.cast::<u8>()) as usize }
        }
    }

    pub use section::Bounds;
}

/// Define a link section.
///
/// The definition site generates two items: a static section struct that is
/// used to access the section, and a macro that is used to place items into the
/// section. The macro is used by the [`in_section`] procedural macro.
///
/// # Attributes
///
/// - `no_macro`: Does not generate the submission macro at the definition site.
///   This will require any associated [`in_section`] invocations to use the raw
///   name of the section.
/// - `aux = <name>`: Specifies that this section is an auxiliary section, and
///   that the section is named `<name>+<aux>`.
///
/// # Example
/// ```rust
/// use link_section::{in_section, section};
///
/// #[section]
/// pub static DATA_SECTION: link_section::Section;
///
/// #[in_section(DATA_SECTION)]
/// pub fn data_function() {
///     println!("data_function");
/// }
/// ```
#[cfg(feature = "proc_macro")]
pub use ::linktime_proc_macro::section;

/// Place an item into a link section.
///
/// # Functions and typed sections
///
/// As a special case, since function declarations by themselves are not sized,
/// functions in typed sections are split and stored as function pointers.
#[cfg(feature = "proc_macro")]
pub use ::linktime_proc_macro::in_section;

/// An untyped link section that can be used to store any type. The underlying
/// data is not enumerable.
#[repr(C)]
pub struct Section {
    name: &'static str,
    bounds: __support::Bounds,
}

impl Section {
    #[doc(hidden)]
    pub const unsafe fn new(name: &'static str, bounds: __support::Bounds) -> Self {
        Self { name, bounds }
    }

    /// The byte length of the section.
    #[inline]
    pub fn byte_len(&self) -> usize {
        self.bounds.byte_len()
    }

    /// The start address of the section.
    #[inline]
    pub fn start_ptr(&self) -> *const () {
        self.bounds.start_ptr()
    }
    /// The end address of the section.
    #[inline]
    pub fn end_ptr(&self) -> *const () {
        self.bounds.end_ptr()
    }
}

impl ::core::fmt::Debug for Section {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Section")
            .field("name", &self.name)
            .field("start", &self.start_ptr())
            .field("end", &self.end_ptr())
            .field("byte_len", &self.byte_len())
            .finish()
    }
}

unsafe impl Sync for Section {}
unsafe impl Send for Section {}

/// A typed link section that can be used to store any sized type. The
/// underlying data is enumerable.
#[repr(C)]
pub struct TypedSection<T: 'static> {
    name: &'static str,
    bounds: __support::Bounds,
    _phantom: ::core::marker::PhantomData<T>,
}

impl<T: 'static> TypedSection<T> {
    #[doc(hidden)]
    pub const unsafe fn new(name: &'static str, bounds: __support::Bounds) -> Self {
        Self {
            name,
            bounds,
            _phantom: ::core::marker::PhantomData,
        }
    }

    /// The start address of the section.
    #[inline(always)]
    pub fn start_ptr(&self) -> *const T {
        self.bounds.start_ptr() as *const T
    }

    /// The end address of the section.
    #[inline(always)]
    pub fn end_ptr(&self) -> *const T {
        self.bounds.end_ptr() as *const T
    }

    /// The start address of the section.
    #[inline]
    pub fn start_ptr_mut(&self) -> *mut T {
        self.bounds.start_ptr() as *mut T
    }

    /// The start address of the section.
    #[inline]
    pub fn end_ptr_mut(&self) -> *mut T {
        self.bounds.end_ptr() as *mut T
    }

    /// The stride of the typed section.
    #[inline(always)]
    pub const fn stride(&self) -> usize {
        assert!(
            ::core::mem::size_of::<T>() > 0
                && ::core::mem::size_of::<T>() * 2 == ::core::mem::size_of::<[T; 2]>()
        );
        ::core::mem::size_of::<T>()
    }

    /// The byte length of the section.
    #[inline]
    pub fn byte_len(&self) -> usize {
        self.bounds.byte_len()
    }

    /// The number of elements in the section.
    #[inline]
    pub fn len(&self) -> usize {
        self.byte_len() / self.stride()
    }

    /// True if the section is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The section as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        if self.is_empty() {
            &[]
        } else {
            unsafe { ::core::slice::from_raw_parts(self.start_ptr(), self.len()) }
        }
    }

    /// The offset of the item in the section, if it is in the section.
    #[inline]
    pub fn offset_of(&self, item: &T) -> Option<usize> {
        let ptr = item as *const T;
        if ptr < self.start_ptr() || ptr >= self.end_ptr() {
            None
        } else {
            Some(unsafe { ptr.offset_from(self.start_ptr()) as usize })
        }
    }

    /// The section as a mutable slice.
    ///
    /// # Safety
    ///
    /// This cannot be safely used and is _absolutely unsound_ if any other
    /// slices are live.
    #[allow(clippy::mut_from_ref)]
    #[inline]
    pub unsafe fn as_mut_slice(&self) -> &mut [T] {
        if self.is_empty() {
            &mut []
        } else {
            unsafe { ::core::slice::from_raw_parts_mut(self.start_ptr() as *mut T, self.len()) }
        }
    }
}

impl<'a, T> ::core::iter::IntoIterator for &'a TypedSection<T> {
    type Item = &'a T;
    type IntoIter = ::core::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<T> ::core::ops::Deref for TypedSection<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> ::core::fmt::Debug for TypedSection<T> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("TypedSection")
            .field("name", &self.name)
            .field("start", &self.start_ptr())
            .field("end", &self.end_ptr())
            .field("len", &self.len())
            .field("stride", &self.stride())
            .finish()
    }
}

unsafe impl<T> Sync for TypedSection<T> where T: Sync {}
unsafe impl<T> Send for TypedSection<T> where T: Send {}
