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
mod meta;
mod platform;
mod section_parse;
mod sections;

pub use item::SectionItemLocation;
pub use sections::{
    MovableBackref, MovableRef, Ref, Section, TypedMovableSection, TypedMutableSection,
    TypedReferenceSection, TypedSection,
};

/// Types for [`TypedReferenceSection`].
#[deprecated(since = "0.17.1", note = "Use [`Ref`] from the crate root instead.")]
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
        example: "untyped | typed | mutable | movable | reference";
        validate: [(untyped), (typed), (mutable), (movable), (reference)];
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
        attr: [(aux(main = $aux_str:ident)) => ($aux_str)];
        example: "aux(main = MAIN_SECTION)";
    };
    name {
        attr: [(name = $name_str:ident) => ($name_str)];
        example: "name = SECTION";
    };
    section_type {
        attr: [(type = $section_type_name:ident) => ($section_type_name)];
        example: "type = untyped | typed | mutable | movable | reference";
        validate: [(untyped), (typed), (mutable), (movable), (reference)];
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

    #[doc(hidden)]
    #[macro_export]
    macro_rules! __hash_no_proc_macro {
        ((__) (($($__prefix:literal $(,)?)*)) ($($name:tt)*) (($($__suffix:literal $(,)?)*)) $__hash_length:literal $__max_length:literal $__valid_section_chars:literal) => {
            concat!($($__prefix,)* $(stringify!($name)),* $(,$__suffix)*);
        };
    }
    #[cfg(not(feature = "proc_macro"))]
    pub use __hash_no_proc_macro as hash;

    #[cfg(miri)]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __address_of_symbol {
        ($ref_or_item:ident $section:ident $type:ident $name:ident $($aux:ident)?) => {
            // Miri does not support any of these linker-defined extern statics
            // see: https://github.com/rust-lang/miri/blob/master/src/shims/extern_static.rs#L15
            ::core::ptr::null() as *const ()
        };
    }

    #[cfg(not(miri))]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __address_of_symbol {
        ($ref_or_item:ident $section:ident $type:ident $name:ident $($aux:ident)?) => {
            {
                // These are not valid items, but they are valid pointers.
                // We cannot safely use them - only take pointers to them.
                $crate::__add_linktime_attributes_to_static!(
                    extern "C" {
                        #[link_name = $crate::__support::section_name!(string $ref_or_item $section $type $name $($aux)?)]
                        static __SYMBOL: u8;
                    }
                );
                // TODO: black_box when hint is stable
                // TODO: MSRV: we can use &raw const once we bump MSRV
                // unsafe { &raw const __SYMBOL as *const () }
                unsafe { ::core::ptr::addr_of!(__SYMBOL) as *const () }
            }
        };
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! __add_section_link_attribute(
        ($ref_or_item:ident $section:ident $type:ident $name:ident $($aux:ident)? #[$attr:ident = __]
            $(#[$meta:meta])*
            $vis:vis static $($static:tt)*
        ) => {
            $crate::__add_linktime_attributes_to_static!(
                #[$attr = $crate::__support::section_name!(string $ref_or_item $section $type $name $($aux)?)]
                $(#[$meta])*
                $vis static $($static)*
            );
        };
        ($ref_or_item:ident $section:ident $type:ident $name:ident $($aux:ident)? #[$attr:ident = __]
            extern "C" {
                $(#[$meta:meta])*
                $vis:vis static $($static:tt)*
            }
        ) => {
            $crate::__add_linktime_attributes_to_static!(
                extern "C" {
                    #[link_name = $crate::__support::section_name!(string $ref_or_item $section $type $name $($aux)?)]
                    $(#[$meta])*
                    $vis static $($static)*
                }
            );
        };
        ($ref_or_item:ident $section:ident $type:ident $name:ident $($aux:ident)? #[$attr:ident = __]
            $($item:tt)*) => {
            $crate::__add_linktime_attributes_to_static!(
                #[$attr = $crate::__support::section_name!(string $ref_or_item $section $type $name $($aux)?)]
                $($item)*
            );
        };
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
    macro_rules! __in_section_helper_macro_generic_movable {
        (($($args:tt)*)) => { $crate::__support::in_section_crate!((@v=0 ; (source=section) ; (type=movable) ; $($args)*)); }
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
        ((@v=0 ; (source=$source:ident) ; (type = untyped) $(; (aux = $aux:ident))? ; (section = $section:tt) $(; (path = $path:path))? ; (meta = $meta:tt) ; (item = $item:tt))) => {
            $crate::__in_section_crate!(@untyped $section, $($aux)?, $($path)?, $meta $item);
        };
        ((@v=0 ; (source=$source:ident) ; (type = $section_type:ident) $(; (aux = $aux:ident))? ; (section = $section:tt) $(; (path = $path:path))? ; (meta = $meta:tt) ; (item = $item:tt))) => {
            $crate::__in_section_crate!(@typed[$section_type] $section, $($aux)?, $($path)?, $meta $item);
        };

        // Untyped items are placed in the data or code section as-is.
        (@untyped $section:tt, $($aux:ident)?, $($path:path)?, ($($meta:tt)*) ($vis:vis fn $($rest:tt)*)) => {
            $crate::__add_section_link_attribute!(
                item code section $section $($aux)?
                #[link_section = __]
                $($meta)*
                $vis fn $($rest)*
            );
            $(
                const _: () = {
                    $crate::Section::__validate(&$path);
                };
            )?
        };
        (@untyped $section:tt, $($aux:ident)?, $($path:path)?, ($($meta:tt)*) ($($rest:tt)*)) => {
            $crate::__add_section_link_attribute!(
                item data section $section $($aux)?
                #[link_section = __]
                $($meta)*
                $($rest)*
            );
            $(
                const _: () = {
                    $crate::Section::__validate(&$path);
                };
            )?
        };

        // Convert fn() with a body to a const item and a function pointer item.
        (@typed[$section_type:ident] $section:tt, $($aux:ident)?, $($path:path)?, ($($meta:tt)*) ($vis:vis fn $ident_fn:ident($($args:tt)*) $(-> $ret:ty)? { $($body:tt)* })) => {
            $($meta)*
            $vis fn $ident_fn($($args)*) $(-> $ret)? {
                $crate::__in_section_crate!(@typed[$section_type] $section, $($aux)?, $($path)?, () (
                    const _: fn($($args)*) $(-> $ret)? = $ident_fn;
                ));

                $($body)*
            }
        };

        // If no path is provided, use the item type.
        (@typed[$section_type:ident] $section:tt, $($aux:ident)?, , $meta:tt ($vis:vis $const_or_static:ident $name:tt : $ty:ty = $($rest:tt)*)) => {
            $crate::__in_section_crate!(@typed[$section_type] $section, $($aux)?, $crate::TypedSection::<$ty>, $meta (
                $vis $const_or_static $name: $ty = $($rest)*
            ));
        };

        (@type_select $path:path) => {
            <$path as $crate::__support::SectionItemType>::Item
        };

        // static items
        (@typed[typed] $section:tt, $($aux:ident)?, $path:path, ($($meta:tt)*) ($vis:vis static $ident:ident : $ty:ty = $value:expr;)) => {
            #[cfg(not(target_family = "wasm"))]
            $crate::__add_section_link_attribute!(
                item data section $section $($aux)?
                #[link_section = __]
                $($meta)*
                $vis static $ident: $crate::__in_section_crate!(@type_select $path) = const {
                    const _: () = {
                        let _: *const <$path as $crate::__support::SectionItemTyped<$ty>>::Item = ::core::ptr::null();
                    };

                    $value
                };
            );

            #[cfg(target_family = "wasm")]
            compile_error!("static items are not supported on WASM: use const items instead");
        };

        // mutable const items live in SyncUnsafeCell
        (@typed[mutable] $section:tt, $($aux:ident)?, $path:path, ($($meta:tt)*) ($vis:vis const $ident:tt: $ty:ty = $value:expr;)) => {
            $($meta)*
            $vis const $ident: $ty = const {
                type __InSecStoredTy = $crate::__in_section_crate!(@type_select $path);
                const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = $value;

                $crate::__register_wasm_item!(mutable, value=__LINK_SECTION_CONST_ITEM_VALUE, section=$section $($aux)?);

                #[cfg(not(target_family = "wasm"))]
                $crate::__add_section_link_attribute!(
                    item data section $section $($aux)?
                    #[link_section = __]
                    static __LINK_SECTION_CONST_ITEM: $crate::__support::SyncUnsafeCell<__InSecStoredTy> = $crate::__support::SyncUnsafeCell::new(__LINK_SECTION_CONST_ITEM_VALUE);
                );

                __LINK_SECTION_CONST_ITEM_VALUE
            };
        };

        (@typed[mutable] $($rest:tt)*) => {
            compile_error!("Only const items are supported in mutable sections");
        };

        // movable static items expose a MovableRef and submit hidden value/backref records.
        (@typed[movable] $section:tt, $($aux:ident)?, $path:path, ($($meta:tt)*) ($vis:vis static $ident:ident: $ty:ty = $value:expr;)) => {
            $($meta)*
            $vis static $ident: $crate::MovableRef<$crate::__in_section_crate!(@type_select $path)> = const {
                const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = $value;
                type __InSecStoredTy = $crate::__in_section_crate!(@type_select $path);
                #[cfg(not(target_family = "wasm"))]
                {
                    $crate::__add_section_link_attribute!(
                        item data section $section $($aux)?
                        #[link_section = __]
                        static __LINK_SECTION_CONST_ITEM: $crate::__support::SyncUnsafeCell<__InSecStoredTy> =
                            $crate::__support::SyncUnsafeCell::new(__LINK_SECTION_CONST_ITEM_VALUE);
                    );

                    $crate::__add_section_link_attribute!(
                        backref data section $section $($aux)?
                        #[link_section = __]
                        static __LINK_SECTION_MOVABLE_BACKREF: $crate::__support::SyncUnsafeCell<
                            $crate::MovableBackref<__InSecStoredTy>
                        > = $crate::__support::SyncUnsafeCell::new(
                            $crate::MovableBackref::new(
                                $crate::MovableRef::slot_ptr(&raw const $ident),
                            )
                        );
                    );

                    $crate::MovableRef::new(
                        (&raw const __LINK_SECTION_CONST_ITEM)
                            .cast::<__InSecStoredTy>(),
                    )
                }

                #[cfg(target_family = "wasm")]
                {
                    $crate::__register_wasm_item!(
                        movable,
                        value=__LINK_SECTION_CONST_ITEM_VALUE,
                        slot=$crate::MovableRef::slot_ptr(&raw const $ident),
                        section=$section $($aux)?
                    );

                    $crate::MovableRef::new(::core::ptr::null())
                }
            };
        };

        (@typed[movable] $($rest:tt)*) => {
            compile_error!("Only static items are supported in movable sections");
        };

        // const items are the same across all other types
        (@typed[$section_type:ident] $section:tt, $($aux:ident)?, $path:path, ($($meta:tt)*) ($vis:vis const $ident:tt: $ty:ty = $value:expr;)) => {
            $($meta)*
            $vis const $ident: $ty = const {
                type __InSecStoredTy = $crate::__in_section_crate!(@type_select $path);
                const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = $value;

                $crate::__register_wasm_item!($section_type, value=__LINK_SECTION_CONST_ITEM_VALUE, section=$section $($aux)?);

                #[cfg(not(target_family = "wasm"))]
                $crate::__add_section_link_attribute!(
                    item data section $section $($aux)?
                    #[link_section = __]
                    static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = __LINK_SECTION_CONST_ITEM_VALUE;
                );

                __LINK_SECTION_CONST_ITEM_VALUE
            };
        };

        (@typed[reference] $section:tt, $($aux:ident)?, $path:path, ($($meta:tt)*) ($vis:vis static $ident:ident: $ty:ty = $value:expr;)) => {
            #[cfg(target_family="wasm")]
            $($meta)*
            $vis static $ident: $crate::reference::Ref<$crate::__in_section_crate!(@type_select $path)> = {
                type __InSecStoredTy = $crate::__in_section_crate!(@type_select $path);
                const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = $value;
                $crate::__register_wasm_item!(reference, value=__LINK_SECTION_CONST_ITEM_VALUE, ref=$ident, section=$section $($aux)?);
                $crate::reference::Ref::new()
            };

            // On non-WASM platforms, we can store the value directly (repr(transparent) allows this).
            #[cfg(not(target_family="wasm"))]
            $crate::__add_section_link_attribute!(
                item data section $section $($aux)?
                #[link_section = __]
                $($meta)*
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
/// - `aux(main = <name>)`: Specifies that this section is an auxiliary section, and
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
///
/// ## Raw items
///
/// This macro can place items into a section that is not normally visible to it
/// by using `#[in_section(unsafe, type = typed|movable|..., name =
/// SECTION_NAME, ...)`. Raw items are not validated at compile time, and must
/// be validated by the author.
#[cfg(feature = "proc_macro")]
pub use ::linktime_proc_macro::in_section;
