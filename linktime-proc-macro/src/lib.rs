#![doc = include_str!("../README.md")]

mod generate;
mod tokens;

use proc_macro::TokenStream;

/// Generates macros in the low-level crate and the linktime crate.
macro_rules! generators {
    ($(
        ($crate_name:ident: $( $macro_name:ident ),*) => {
            $(
                #[allow(missing_docs)]
                #[proc_macro_attribute]
                pub fn $macro_name(attribute: TokenStream, item: TokenStream) -> TokenStream {
                    generate($macro_name, $crate_name, attribute, item)
                }
            )*
            mod linktime {
                $(
                    #[allow(missing_docs)]
                    #[proc_macro_attribute]
                    pub fn $macro_name(attribute: TokenStream, item: TokenStream) -> TokenStream {
                        generate($macro_name, "linktime", attribute, item)
                    }
                )*
            }
        };
    )*)
}

generators! {
    (ctor: ctor)
    (dtor: dtor)
    (link_section: in_section, section: section)
}
