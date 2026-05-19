#![doc = include_str!("../README.md")]

pub mod hash;
pub mod iterable;
pub mod map;
pub mod referenced_slice;
pub mod slice;
pub mod sorted_referenced_slice;
pub mod sorted_slice;

pub use iterable::ScatteredIterable;
pub use map::ScatteredMap;
pub use referenced_slice::ScatteredReferencedSlice;
pub use slice::ScatteredSlice;
pub use sorted_referenced_slice::ScatteredSortedReferencedSlice;
pub use sorted_slice::ScatteredSortedSlice;

#[doc(hidden)]
#[macro_export]
macro_rules! __scatter_parse {
    // Send the #[scatter]'d item into the collection's private macro.
    (#[scatter ($($meta:tt)*)] $(#[$imeta:meta])* $($item:tt)*) => {
        $($meta)* ! (
            ($($meta)* =>
            $(#[$imeta])*
            $($item)*
            )
        );
    };
    (#[scatter] $($rest:tt)* ) => {
        compile_error!("Unknown collection type");
    };

    (@reorder (#[scatter] $($item:tt)*) ($($rest:tt)*)) => {
        $crate::__support::scatter_parse!(#[scatter] $($rest)* $($item)*);
    };
    (@reorder (#[$top:meta] $($item:tt)*) ($($rest:tt)*)) => {
        $crate::__support::scatter_parse!(@reorder($($item)*) (#[$top] $($rest)*));
    };
    (@reorder ($item:item;) $($rest:tt)*) => {
        compile_error!("Missing #[scatter] attribute.");
    };
    (@reorder $($rest:tt)*) => {
        compile_error!("Missing #[scatter] attribute.");
    };

    ($($rest:tt)*) => {
        $crate::__support::scatter_parse!(@reorder ($($rest)*) ());
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __gather_parse {
    // Send the #[gather]'d item into the collection's private macro.
    (@done ($collection:ident) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredSlice < $ty:ty >; ) => {
        $crate::__slice ! (
            @gather
            $(#[$imeta])*
            $vis static $name: $collection < $ty >;
        );
    };

    (@done ($collection:ident) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredSortedSlice < $ty:ty >; ) => {
        $crate::__sorted_slice ! (
            @gather
            $(#[$imeta])*
            $vis static $name: $collection < $ty >;
        );
    };

    (@done ($collection:ident) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredReferencedSlice < $ty:ty >; ) => {
        $crate::__referenced_slice ! (
            @gather
            $(#[$imeta])*
            $vis static $name: $collection < $ty >;
        );
    };

    (@done ($collection:ident) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredSortedReferencedSlice < $ty:ty >; ) => {
        $crate::__sorted_referenced_slice ! (
            @gather
            $(#[$imeta])*
            $vis static $name: $collection < $ty >;
        );
    };

    (@done ($map:ident) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredMap < $key:ty, $value:ty >; ) => {
        $crate::__map ! (
            @gather
            $(#[$imeta])*
            $vis static $name: $map < $key, $value >;
        );
    };

    (@done ($collection:ident) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredIterable < $ty:ty >; ) => {
        $crate::__iterable ! (
            @gather
            $(#[$imeta])*
            $vis static $name: $collection < $ty >;
        );
    };

    (@done #[gather] $($rest:tt)* ) => {
        compile_error!("Unknown collection type");
    };

    (@reorder (#[gather] $($item:tt)*) ($($rest:tt)*) $collection:tt) => {
        $crate::__support::gather_parse!(@done $collection #[gather] $($rest)* $($item)*);
    };
    (@reorder (#[$top:meta] $($item:tt)*) ($($rest:tt)*) $collection:tt) => {
        $crate::__support::gather_parse!(@reorder ($($item)*) (#[$top] $($rest)*) $collection);
    };
    (@reorder ($item:item;) $($rest:tt)*) => {
        compile_error!("Missing #[gather] attribute.");
    };
    (@reorder $($rest:tt)*) => {
        compile_error!("Missing #[gather] attribute.");
    };

    ($(#$meta:tt)* $vis:vis static $name:ident: $collection:ident < $($rest:tt)* ) => {
        $crate::__support::gather_parse!(@reorder ($(#$meta)* $vis static $name: $collection < $($rest)*) () ($collection));
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
