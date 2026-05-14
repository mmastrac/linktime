#![doc = include_str!("../docs/BUILD.md")]
//! # link-section
#![doc = include_str!("../docs/PREAMBLE.md")]
#![allow(unsafe_code)]
#![cfg_attr(linktime_used_linker, doc(test(attr(feature(used_with_arg)))))]
#![no_std]

#[doc = include_str!("../docs/LIFE_BEFORE_MAIN.md")]
pub mod life_before_main {}

mod item;
mod macros;
mod platform;
mod section_parse;
mod sections;

pub use sections::{Section, TypedMutableSection, TypedReferenceSection, TypedSection};
pub mod reference {
    pub use crate::sections::Ref;
}

__declare_features!(
    section: __section_features;

    @default: type;

    /// Auxiliary sections are stored in a section near the main section. The
    /// aux path must be a valid reference to the main section.
    aux {
        attr: [(aux(main = $($aux_name:tt)*)) => (($($aux_name)*))];
        example: "aux(main = path::to::MAIN_SECTION)";
        validate: [(($aux_name:path))];
    };
    /// Specify a custom crate path for the `link-section` crate. Used when
    /// re-exporting the section macro.
    crate_path {
        attr: [(crate_path = $path:pat) => (($path))];
        example: "crate_path = ::path::to::link_section";
    };
    /// Define a custom macro name for the submission macro. Note that this is
    /// required when not using the `proc_macro` feature and using `aux`.
    macro_unique_name {
        attr: [(macro_unique_name = $macro_unique_name_str:ident) => (($macro_unique_name_str))];
        example: "macro_unique_name = my_unique_name_1234";
    };
    /// Disable submission macro generation at the definition site.
    no_macro {
        attr: [(no_macro) => (no_macro)];
    };
    /// Crate feature `proc_macro` (enables the `#[section]` attribute shim).
    proc_macro {
        feature: "proc_macro";
    };
    /// The type of the section.
    type {
        attr: [
            (type = $section_type:ident) => ($section_type)
        ];
        example: "untyped | typed | mutable | reference";
        validate: [(untyped), (typed), (mutable), (reference)];
    };
);

#[cfg(doc)]
__generate_docs!(__section_features);

__declare_features!(
    in_section: __in_section_features;

    @default: section;

    section {
        attr: [(section = $($section_path:tt)*) => (($($section_path)*))];
        example: "[section = ] ::path::to::SECTION";
        validate: [(($section_path:path))];
    };
    aux {
        attr: [(aux = $aux_str:ident) => ($aux_str)];
        example: "aux = MAIN_SECTION";
    };
    name {
        attr: [(name = $name_str:ident) => ($name_str)];
        example: "name = SECTION";
    };
    section_type {
        attr: [(type = $section_type_name:ident) => ($section_type_name)];
        example: "type = untyped | typed | mutable | reference";
        validate: [(untyped), (typed), (mutable), (reference)];
    };
    unsafe {
        attr: [(unsafe) => (unsafe)];
    };
);

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
    pub use crate::__in_section_crate as in_section_crate;
    pub use crate::__in_section_parse as in_section_parse;
    pub use crate::__section_parse as section_parse;

    pub use crate::sections::IsUntypedSection;
    pub use crate::{item::*, platform::*};

    #[cfg(feature = "proc_macro")]
    pub use linktime_proc_macro::hash;
    #[cfg(feature = "proc_macro")]
    pub use linktime_proc_macro::ident_concat;

    #[cfg(not(linktime_used_linker))]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __add_used {
        (
            $section:ident $type:ident $name:ident $($aux:ident)? #[$attr:ident = __]
            $(#[$meta:meta])*
            $vis:vis static $ident:ident : $($static:tt)*
        ) => {
            $crate::__add_section_link_attribute_impl!(
                $section $type $name $($aux)? #[$attr = __]
                $(#[$meta])*
                #[used]
                #[cfg_attr(target_os = "aix", export_name = concat!("_", env!("CARGO_PKG_NAME"), "_",
                    ::core::module_path!(), "_",
                    stringify!($ident),
                    "_L", line!(), "C", column!()))]
                $vis static $ident : $($static)*
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
            $vis:vis static $ident:ident : $($static:tt)*
        ) => {
            $crate::__add_section_link_attribute_impl!(
                $section $type $name $($aux)? #[$attr = __]
                $(#[$meta])*
                #[used(linker)]
                #[cfg_attr(target_os = "aix", export_name = concat!("_", env!("CARGO_PKG_NAME"), "_",
                    ::core::module_path!(), "_",
                    stringify!($ident),
                    "_L", line!(), "C", column!()))]
                $vis static $ident : $($static)*
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

    // Without the proc macro, only name/type supported (no `aux`).
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __declare_macro {
        ($vis:vis $ident:ident $generic_macro:ident) => {
            /// Internal macro for parsing the section. This is exported with
            /// the same name as the type below.
            #[doc(hidden)]
            $vis use $crate::$generic_macro as $ident;
        };
    }

    #[macro_export]
    #[doc(hidden)]
    macro_rules! __in_section_helper_macro_generic {
        (($($args:tt)*)) => { $crate::__support::in_section_crate!((@v=0 ; (source=section) ; (type=typed) ; $($args)*)); }
    }

    #[macro_export]
    #[doc(hidden)]
    macro_rules! __in_section_helper_macro_generic_mutable {
        (($($args:tt)*)) => { $crate::__support::in_section_crate!((@v=0 ; (source=section) ; (type=mutable) ; $($args)*)); }
    }

    #[macro_export]
    #[doc(hidden)]
    macro_rules! __in_section_helper_macro_generic_reference {
        (($($args:tt)*)) => { $crate::__support::in_section_crate!((@v=0 ; (source=section) ; (type=reference) ; $($args)*)); }
    }

    #[macro_export]
    #[doc(hidden)]
    macro_rules! __in_section_helper_macro_no_generic {
        (($($args:tt)*)) => { $crate::__support::in_section_crate!((@v=0 ; (source=section) ; (type=untyped) ; $($args)*)); }
    }

    #[macro_export]
    #[doc(hidden)]
    #[allow(unknown_lints, edition_2024_expr_fragment_specifier)]
    macro_rules! __in_section_crate {
        ((@v=0 ; (source=$source:ident) ; (type = untyped) $(; (aux = $aux:ident))? ; (section = $section:tt) $(; (path = $path:path))? ; (item = $item:tt))) => {
            $crate::__in_section_crate!(@untyped $section, $($aux)?, $($path)?, $item);
        };
        ((@v=0 ; (source=$source:ident) ; (type = $section_type:ident) $(; (aux = $aux:ident))? ; (section = $section:tt) $(; (path = $path:path))? ; (item = $item:tt))) => {
            $crate::__in_section_crate!(@typed[$section_type] $section, $($aux)?, $($path)?, $item);
        };

        // Untyped items are placed in the data or code section as-is.
        (@untyped $section:tt, $($aux:ident)?, $($path:path)?, ($(#[$meta:meta])* $vis:vis fn $($rest:tt)*)) => {
            $(
                const _: () = {
                    $crate::Section::__validate(&$path);
                };
            )?
            $crate::__add_section_link_attribute!(
                code section $section $($aux)?
                #[link_section = __]
                $(#[$meta])*
                $vis fn $($rest)*
            );
        };
        (@untyped $section:tt, $($aux:ident)?, $($path:path)?, ($($rest:tt)*)) => {
            $(
                const _: () = {
                    $crate::Section::__validate(&$path);
                };
            )?
            $crate::__add_section_link_attribute!(
                data section $section $($aux)?
                #[link_section = __]
                $($rest)*
            );
        };

        // Convert fn() with a body to a const item and a function pointer item.
        (@typed[$section_type:ident] $section:tt, $($aux:ident)?, $($path:path)?, ($(#[$meta:meta])* $vis:vis fn $ident_fn:ident($($args:tt)*) $(-> $ret:ty)? $body:block)) => {
            $crate::__in_section_crate!(@typed[$section_type] $section, $($aux)?, $($path)?, (
                const _: fn($($args)*) $(-> $ret)? = $ident_fn;
            ));

            $(#[$meta])*
            $vis fn $ident_fn($($args)*) $(-> $ret)? $body
        };

        // If no path is provided, use the item type.
        (@typed[$section_type:ident] $section:tt, $($aux:ident)?, , ($(#[$meta:meta])* $vis:vis $const_or_static:ident $name:tt : $ty:ty = $($rest:tt)*)) => {
            $crate::__in_section_crate!(@typed[$section_type] $section, $($aux)?, $crate::TypedSection::<$ty>, (
                $(#[$meta])*
                $vis $const_or_static $name: $ty = $($rest)*
            ));
        };

        (@type_select $path:path) => {
            <$path as $crate::__support::SectionItemType>::Item
        };

        // static items
        (@typed[typed] $section:tt, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis static $ident:ident : $ty:ty = $value:expr;)) => {
            #[cfg(target_family = "wasm")]
            compile_error!("static items are not supported on WASM: use const items instead");

            const _: () = {
                let _: *const <$path as $crate::__support::SectionItemTyped<$ty>>::Item = ::core::ptr::null();
            };

            #[cfg(not(target_family = "wasm"))]
            $crate::__add_section_link_attribute!(
                data section $section $($aux)?
                #[link_section = __]
                $(#[$meta])*
                $vis static $ident: $crate::__in_section_crate!(@type_select $path) = $value;
            );
        };

        // const items with a name are the same across all types
        (@typed[$section_type:ident] $section:tt, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis const $ident:ident: $ty:ty = $value:expr;)) => {
            $(#[$meta])* $vis const $ident: $ty = {
                type __InSecStoredTy = $crate::__in_section_crate!(@type_select $path);
                const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = $value;

                #[cfg(target_family = "wasm")]
                {
                    // Register a counting item
                    $crate::__add_section_link_attribute!(
                        data section $section $($aux)?
                        #[link_section = __]
                        $vis static __LINK_SECTION_CONST_ITEM: u8 = 0;
                    );

                    $crate::__add_section_link_attribute!(
                        data bounds $section $($aux)?
                        #[link_name = __]
                        extern "C" {
                            static __LINK_SECTION_INFO: $crate::__support::wasm::LinkSectionRawInfo;
                        }
                    );

                    #[link_section = ".init_array.0"]
                    static __LINK_SECTION_ITEM_FN_REF: extern "C" fn() = {
                        extern "C" fn __LINK_SECTION_ITEM_FN() {
                            static DISARMED: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);
                            if DISARMED.swap(true, ::core::sync::atomic::Ordering::Relaxed) {
                                return;
                            }
                            unsafe {
                                let ptr = $crate::__support::wasm::register_wasm_link_section_item(&raw const __LINK_SECTION_INFO);
                                ::core::ptr::write(ptr as *mut __InSecStoredTy, __LINK_SECTION_CONST_ITEM_VALUE);
                            }
                        }
                        __LINK_SECTION_ITEM_FN
                    };
                }

                #[cfg(not(target_family = "wasm"))]
                $crate::__add_section_link_attribute!(
                    data section $section $($aux)?
                    #[link_section = __]
                    $(#[$meta])*
                    $vis static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = __LINK_SECTION_CONST_ITEM_VALUE;
                );

                __LINK_SECTION_CONST_ITEM_VALUE
            };
        };

        // anonymous const items are the same across all types
        (@typed[$section_type:ident] $section:tt, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis const _: $ty:ty = $value:expr;)) => {
            $(#[$meta])* $vis const _: () = {
                type __InSecStoredTy = $crate::__in_section_crate!(@type_select $path);
                $crate::__add_section_link_attribute!(
                    data section $section $($aux)?
                    #[link_section = __]
                    $(#[$meta])*
                    $vis static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = $value;
                );
            };
        };

        (@typed[reference] $section:tt, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis static $ident:ident: $ty:ty = $value:expr;)) => {
            #[cfg(target_family="wasm")]
            $vis static $ident: $crate::reference::Ref<$crate::__in_section_crate!(@type_select $path)> = const {
                type __InSecStoredTy = $crate::__in_section_crate!(@type_select $path);

                // Register a counting item
                $crate::__add_section_link_attribute!(
                    data section $section $($aux)?
                    #[link_section = __]
                    $vis static __LINK_SECTION_CONST_ITEM: u8 = 0;
                );

                $crate::__add_section_link_attribute!(
                    data bounds $section $($aux)?
                    #[link_name = __]
                    extern "C" {
                        static __LINK_SECTION_INFO: $crate::__support::wasm::LinkSectionRawInfo;
                    }
                );

                #[link_section = ".init_array.0"]
                static __LINK_SECTION_ITEM_FN_REF: extern "C" fn() = {
                    extern "C" fn __LINK_SECTION_ITEM_FN() {
                        static DISARMED: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);
                        if DISARMED.swap(true, ::core::sync::atomic::Ordering::Relaxed) {
                            return;
                        }
                        const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = $value;
                        unsafe {
                            let ptr = $crate::__support::wasm::register_wasm_link_section_item(&raw const __LINK_SECTION_INFO);
                            ::core::ptr::write(ptr as *mut __InSecStoredTy, __LINK_SECTION_CONST_ITEM_VALUE);
                            $ident.set(ptr);
                        }
                    }
                    __LINK_SECTION_ITEM_FN
                };

                $crate::reference::Ref::new()
            };

            // On non-WASM platforms, we can store the value directly (repr(transparent) allows this).
            #[cfg(not(target_family="wasm"))]
            $crate::__add_section_link_attribute!(
                data section $section $($aux)?
                #[link_section = __]
                $(#[$meta])*
                $vis static $ident: $crate::reference::Ref<$crate::__in_section_crate!(@type_select $path)> = $crate::reference::Ref::new($value);
            );
        };

        ($($input:tt)*) => {
            compile_error!(concat!("Unexpected input to __in_section_crate: ", stringify!($($input)*)));
        };
    }
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
/// #[section(untyped)]
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
