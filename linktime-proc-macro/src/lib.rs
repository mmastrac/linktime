#![doc = include_str!("../README.md")]

mod generate;
mod hash;
mod tokens;

use proc_macro::TokenStream;

/// Generates macros in the low-level crate and the linktime crate.
macro_rules! generators {
    ( $( ($crate_name:ident/$crate_name_str:literal: $( $macro_name:ident/$macro_name_linktime:ident ),*) )* ) => {
        $($(
            #[cfg(feature = $crate_name_str)]
            #[allow(missing_docs)]
            #[doc(hidden)]
            #[proc_macro_attribute]
            pub fn $macro_name(attribute: TokenStream, item: TokenStream) -> TokenStream {
                crate::generate::generate(stringify!($crate_name), stringify!($macro_name), attribute, item)
            }
        )*)*
        $($(
            #[cfg(feature = $crate_name_str)]
            #[allow(missing_docs)]
            #[doc(hidden)]
            #[proc_macro_attribute]
            pub fn $macro_name_linktime(attribute: TokenStream, item: TokenStream) -> TokenStream {
                crate::generate::generate("linktime", stringify!($macro_name), attribute, item)
            }
        )*)*
    };
}

generators! {
    (ctor/"ctor": ctor/ctor_linktime)
    (dtor/"dtor": dtor/dtor_linktime)
    (link_section/"link_section": in_section/in_section_linktime, section/section_linktime)
}

#[doc(hidden)]
#[proc_macro]
pub fn ident_concat(item: TokenStream) -> TokenStream {
    hash::ident_concat(item)
}

#[doc(hidden)]
#[proc_macro]
pub fn hash(item: TokenStream) -> TokenStream {
    hash::hash(item)
}
