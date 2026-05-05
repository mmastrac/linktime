#[doc(hidden)]
#[macro_export]
macro_rules! __scatter_parse {
    (#[scatter $(($($meta:tt)*))?] $(#[$imeta:meta])* $($item:tt)*) => {
        $(#[$imeta])*
        $($item)*
    };
    (#[$imeta:meta] $($rest:tt)*) => {
        $crate::__support::scatter_parse!(__reorder__(#[$imeta],), $($rest)*);
    };
    (__reorder__($(#[$imeta:meta],)*), #[scatter $(($($meta:tt)*))?] $($rest:tt)*) => {
        $crate::__support::scatter_parse!(#[scatter $(($($meta)*))?] $(#[$imeta])* $($rest)*);
    };
    (__reorder__($(#[$imeta:meta],)*), #[$imeta2:meta] $($rest:tt)*) => {
        $crate::__support::scatter_parse!(__reorder__($(#[$imeta],)*#[$imeta2],), $($rest)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __gather_parse {
    (#[gather $(($($meta:tt)*))?] $(#[$imeta:meta])* $($item:tt)*) => {
        $(#[$imeta])*
        $($item)*
    };
    (#[$imeta:meta] $($rest:tt)*) => {
        $crate::__support::gather_parse!(__reorder__(#[$imeta],), $($rest)*);
    };
    (__reorder__($(#[$imeta:meta],)*), #[gather $(($($meta:tt)*))?] $($rest:tt)*) => {
        $crate::__support::gather_parse!(#[gather $(($($meta)*))?] $(#[$imeta])* $($rest)*);
    };
    (__reorder__($(#[$imeta:meta],)*), #[$imeta2:meta] $($rest:tt)*) => {
        $crate::__support::gather_parse!(__reorder__($(#[$imeta],)*#[$imeta2],), $($rest)*);
    };
}

#[doc(hidden)]
#[allow(unused)]
pub mod __support {
    pub use crate::__gather_parse as gather_parse;
    pub use crate::__scatter_parse as scatter_parse;

    pub use ctor;
    pub use link_section;
    pub use scattered_collect_proc_macro::ident_concat;
}

pub mod declarative {
    pub use crate::__gather_brace as gather;
    pub use crate::__scatter_brace as scatter;
}

#[doc(inline)]
pub use scattered_collect_proc_macro::{gather, scatter};

#[doc(hidden)]
#[macro_export]
macro_rules! __gather_brace {
    ($($item:tt)*) => {
        $crate::__support::gather_parse!(#[gather] $($item)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __scatter_brace {
    ($($item:tt)*) => {
        $crate::__support::scatter_parse!($($item)*);
    };
}
