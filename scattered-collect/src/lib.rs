#![doc = include_str!("../README.md")]

pub mod hash;
pub mod iterable;
pub mod map;
pub mod referenced_slice;
pub mod set;
pub mod slice;
pub mod sorted_referenced_slice;
pub mod sorted_slice;

pub use iterable::ScatteredIterable;
pub use map::ScatteredMap;
pub use referenced_slice::ScatteredReferencedSlice;
pub use set::ScatteredSet;
pub use slice::ScatteredSlice;
pub use sorted_referenced_slice::ScatteredSortedReferencedSlice;
pub use sorted_slice::ScatteredSortedSlice;

#[doc(hidden)]
#[macro_export]
macro_rules! __scatter_parse {
    // Send the #[scatter]'d item into the collection's private macro.
    (#[scatter ($($meta:tt)*)] $(#[$imeta:meta])* $($item:tt)*) => {
        $($meta)* ! (
            ([$($meta)*] =>
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

    (@dispatch $macro:ident $(#[$imeta:meta])* $vis:vis static $name:ident $($rest:tt)* ) => {
        $crate::__support::combine!(output=ident prefix=($crate::__gather_parse!) paren=() paren_prefix=(@unique ) input=(
            H
            __LOCATIONHASH__(of=($(#[$imeta])* $vis static $name $($rest)*))
        ) paren_suffix=($macro $(#[$imeta])* $vis static $name $($rest)*) suffix=(;));
    };

    (@unique $unique:ident $macro:ident $(#[$imeta:meta])* $vis:vis static $name:ident: ($($collection:tt)*) ($($ty:tt)*);) => {
        $crate::$macro!(@gather $unique $(#[$imeta])* $vis static $name: ($($collection)*) <$($ty)*>;);

        $crate::__support::combine!(output=ident prefix=(#[doc(hidden)] #[macro_export] macro_rules!) input=(__ $name __ $macro __ $unique) suffix=({
            ($passthru:tt) => {
                $crate::$macro!(@scatter [$name :: $unique] [$($ty)*] $passthru);
            };
        }));

        $crate::__support::combine!(output=ident prefix=(#[doc(hidden)] $vis use) input=(__ $name __ $macro __ $unique) suffix=(as $name;));
    };

    (@done ($($collection:tt)*) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredSlice < $ty:ty >; ) => {
        $crate::__support::gather_parse!(@dispatch __slice $(#[$imeta])* $vis static $name: ($($collection)*) ( $ty ););
    };

    (@done ($($collection:tt)*) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredSortedSlice < $ty:ty >; ) => {
        $crate::__support::gather_parse!(@dispatch __sorted_slice $(#[$imeta])* $vis static $name: ($($collection)*) ( $ty ););
    };

    (@done ($($collection:tt)*) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredReferencedSlice < $ty:ty >; ) => {
        $crate::__support::gather_parse!(@dispatch __referenced_slice $(#[$imeta])* $vis static $name: ($($collection)*) ( $ty ););
    };

    (@done ($($collection:tt)*) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredSortedReferencedSlice < $ty:ty >; ) => {
        $crate::__support::gather_parse!(@dispatch __sorted_referenced_slice $(#[$imeta])* $vis static $name: ($($collection)*) ( $ty ););
    };

    (@done ($($collection:tt)*) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredMap < $key:ty, $value:ty >; ) => {
        $crate::__support::gather_parse!(@dispatch __map $(#[$imeta])* $vis static $name: ($($collection)*) ( $key, $value ););
    };

    (@done ($($collection:tt)*) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredSet < $key:ty >; ) => {
        $crate::__support::gather_parse!(@dispatch __set $(#[$imeta])* $vis static $name: ($($collection)*) ( $key ););
    };

    (@done ($($collection:tt)*) #[gather] $(#[$imeta:meta])* $vis:vis static $name:ident: ScatteredIterable < $ty:ty >; ) => {
        $crate::__support::gather_parse!(@dispatch __iterable $(#[$imeta])* $vis static $name: ($($collection)*) ( $ty ););
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

    // Chomp through the path until we hit the collection's generics.
    (@path ($($path_part:tt)*) ($(#$meta:tt)* $vis:vis static $name:ident) ($collection:ident < $($rest:tt)*)) => {
        $crate::__support::gather_parse!(@reorder ($(#$meta)* $vis static $name: $collection < $($rest)*) () ($($path_part)* $collection));
    };
    (@path ($($path_part:tt)*) ($(#$meta:tt)* $vis:vis static $name:ident) ($next_path_part:ident $($rest:tt)*)) => {
        $crate::__support::gather_parse!(@path ($($path_part)* $next_path_part) ($(#$meta)* $vis static $name) ($($rest)*));
    };
    (@path ($($path_part:tt)*) ($(#$meta:tt)* $vis:vis static $name:ident) (:: $($rest:tt)*)) => {
        $crate::__support::gather_parse!(@path ($($path_part)* ::) ($(#$meta)* $vis static $name) ($($rest)*));
    };
    (@path ($($path_part:tt)*) ($(#$meta:tt)* $vis:vis static $name:ident) (:: $($rest:tt)*)) => {
        compile_error!("Expected: #[gather] name: path::to::collection<generics>;");
    };

    ($(#$meta:tt)* $vis:vis static $name:ident: $collection:ident < $($rest:tt)* ) => {
        $crate::__support::gather_parse!(@reorder ($(#$meta)* $vis static $name: $collection < $($rest)*) () ($collection));
    };
    ($(#$meta:tt)* $vis:vis static $name:ident: :: $($path:tt)* ) => {
        $crate::__support::gather_parse!(@path (::) ($(#$meta)* $vis static $name) ($($path)*));
    };
    ($(#$meta:tt)* $vis:vis static $name:ident: $path_part:ident $($path:tt)* ) => {
        $crate::__support::gather_parse!(@path ($path_part) ($(#$meta)* $vis static $name) ($($path)*));
    };
}

#[doc(hidden)]
#[allow(unused)]
pub mod __support {
    pub use crate::__gather_parse as gather_parse;
    pub use crate::__scatter_parse as scatter_parse;

    pub use ctor;
    pub use link_section;
    pub use linktime_proc_macro::combine;
}

/// Declarative `scatter!` / `gather!` entry points.
///
/// These are useful for cases where the scattered collections are used within
/// your own macros.
///
/// The syntax is identical to the proc-macro form in both the `scatter` and
/// `gather` cases, with the declarative macro wrapped around them.
///
/// ```rust
/// use scattered_collect::declarative::{scatter, gather};
/// use scattered_collect::slice::ScatteredSlice;
///
/// # fn main() {
/// gather! {
///   #[gather]
///   static ITEMS: ScatteredSlice<u32>;
/// }
///
/// scatter! {
///   #[scatter(ITEMS)]
///   const _: u32 = 1;
/// }
/// # }
/// ```
pub mod declarative {
    /// Declarative form of the `#[gather]` macro.
    ///
    /// Useful for re-exporting these macros from other crates. Wrap this around
    /// the proc-macro form:
    ///
    /// ```rust
    /// # use scattered_collect::declarative::{scatter, gather};
    /// # use scattered_collect::slice::ScatteredSlice;
    ///
    /// # fn main() {
    /// gather! {
    ///   #[gather]
    ///   static ITEMS: ScatteredSlice<u32>;
    /// }
    ///
    /// scatter! {
    ///   #[scatter(ITEMS)]
    ///   const _: u32 = 1;
    /// }
    /// # }
    /// ```
    #[doc(inline)]
    pub use crate::__gather_brace as gather;
    /// Declarative form of the `#[scatter]` macro.
    ///
    /// Useful for re-exporting these macros from other crates. Wrap this around
    /// the proc-macro form:
    ///
    /// ```rust
    /// # use scattered_collect::declarative::{scatter, gather};
    /// # use scattered_collect::slice::ScatteredSlice;
    ///
    /// # fn main() {
    /// gather! {
    ///   #[gather]
    ///   static ITEMS: ScatteredSlice<u32>;
    /// }
    ///
    /// scatter! {
    ///   #[scatter(ITEMS)]
    ///   const _: u32 = 1;
    /// }
    /// # }
    /// ```
    #[doc(inline)]
    pub use crate::__scatter_brace as scatter;
}

#[doc(inline)]
pub use linktime_proc_macro::{gather, scatter};

#[doc(hidden)]
#[macro_export]
macro_rules! __gather_brace {
    ($($item:tt)*) => {
        $crate::__support::gather_parse!($($item)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __scatter_brace {
    ($($item:tt)*) => {
        $crate::__support::scatter_parse!($($item)*);
    };
}
