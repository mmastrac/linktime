pub mod slice;
pub mod sorted_slice;

pub use slice::ScatteredSlice;
pub use sorted_slice::ScatteredSortedSlice;

#[doc(hidden)]
#[macro_export]
macro_rules! __scatter_parse {
    // Send the #[scatter]'d item into the collection's private macro.
    (#[scatter ($($meta:tt)*)] $(#[$imeta:meta])* $($item:tt)*) => {
        $($meta)* ! (
            @scatter [$($meta)*]
            $(#[$imeta])*
            $($item)*
        );
    };

    (#[scatter] $($rest:tt)* ) => {
        compile_error!("Unknown collection type");
    };

    (__reorder__ (#[scatter] $($item:tt)*) ($($rest:tt)*)) => {
        $crate::__support::scatter_parse!(#[scatter] $($rest)* $($item)*);
    };
    (__reorder__ (#[$top:meta] $($item:tt)*) ($($rest:tt)*)) => {
        $crate::__support::scatter_parse!(__reorder__($($item)*) (#[$top] $($rest)*));
    };
    (__reorder__ ($item:item;) $($rest:tt)*) => {
        compile_error!("Missing #[scatter] attribute.");
    };
    (__reorder__ $($rest:tt)*) => {
        compile_error!("Missing #[scatter] attribute.");
    };

    ($($rest:tt)*) => {
        $crate::__support::scatter_parse!(__reorder__ ($($rest)*) ());
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __gather_parse {
    // Send the #[gather]'d item into the collection's private macro.
    (#[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredSlice < $ty:ty >; ) => {
        $crate::__slice ! (
            @gather
            $(#[$imeta])*
            $vis static $name: ScatteredSlice < $ty >;
        );
    };

    (#[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredSortedSlice < $ty:ty >; ) => {
        $crate::__sorted_slice ! (
            @gather
            $(#[$imeta])*
            $vis static $name: ScatteredSortedSlice < $ty >;
        );
    };

    (#[gather] $($rest:tt)* ) => {
        compile_error!("Unknown collection type");
    };

    (__reorder__ (#[gather] $($item:tt)*) ($($rest:tt)*)) => {
        $crate::__support::gather_parse!(#[gather] $($rest)* $($item)*);
    };
    (__reorder__ (#[$top:meta] $($item:tt)*) ($($rest:tt)*)) => {
        $crate::__support::gather_parse!(__reorder__($($item)*) (#[$top] $($rest)*));
    };
    (__reorder__ ($item:item;) $($rest:tt)*) => {
        compile_error!("Missing #[gather] attribute.");
    };
    (__reorder__ $($rest:tt)*) => {
        compile_error!("Missing #[gather] attribute.");
    };

    ($($rest:tt)*) => {
        $crate::__support::gather_parse!(__reorder__ ($($rest)*) ());
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

/// Declarative `scatter!` / `gather!` entry points.
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
