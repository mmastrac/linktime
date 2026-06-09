//! A collection of sized items gathered into a slice in arbitrary link order.
#![doc = concat!("```rust\n", include_str!("../examples/slice.rs"), "\n```\n")]

use link_section::TypedSection;

/// A collection of sized items gathered into a slice in arbitrary link order.
///
/// For a sorted slice without per-item handles, use [`crate::ScatteredSortedSlice`]. For
/// arbitrary link order with `static` handles at each scatter site, use
/// [`crate::ScatteredReferencedSlice`]. For sorted data with stable per-item references,
/// use [`crate::ScatteredSortedReferencedSlice`]. For key lookup, use [`crate::ScatteredMap`].
#[doc = concat!("```rust\n", include_str!("../examples/slice.rs"), "\n```\n")]
pub struct ScatteredSlice<T: 'static> {
    section: &'static TypedSection<T>,
}

impl<T> ScatteredSlice<T> {
    #[doc(hidden)]
    #[allow(unsafe_code)]
    pub const unsafe fn new(section: &'static TypedSection<T>) -> Self {
        Self { section }
    }
}

impl<T> ::core::ops::Deref for ScatteredSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.section
    }
}

/// Declare a scattered slice.
#[macro_export]
#[doc(hidden)]
macro_rules! __slice {
    (@gather $unique:ident $(#[$meta:meta])* $vis:vis static $name:ident: ($($collection:tt)*) < $ty:ty >;) => {
        $(#[$meta])*
        $vis static $name: $($collection)* <$ty> = {
            $crate::__support::link_section::declarative::section!(
                #[section(unsafe, type = typed, name = $name :: $unique)]
                static $name: $crate::__support::link_section::TypedSection<$ty>;
            );

            unsafe { $crate::slice::ScatteredSlice::new(
                $name.const_deref()
            ) }
        };
    };

    (@scatter [$collection_name:ident :: $unique:ident] $types:tt ([$($meta:tt)*] => $(#[$imeta:meta])* $vis:vis $type:ident $name:tt: $ty:ty = $expr:expr;)) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection_name :: $unique, type = typed)]
            $(#[$imeta])*
            $vis $type $name: $ty = $expr;
        );
    };
}

#[cfg(all(test, not(miri)))]
mod tests {
    pub use super::ScatteredSlice;

    __slice!(@gather A pub static TEST_SLICE: (ScatteredSlice)<u32>;);
    __slice!(@scatter [TEST_SLICE :: A] [u32] ([TEST_SLICE] => const _: u32 = 1;));
    __slice!(@scatter [TEST_SLICE :: A] [u32] ([TEST_SLICE] => const _: u32 = 3;));
    __slice!(@scatter [TEST_SLICE :: A] [u32] ([TEST_SLICE] => const _: u32 = 2;));

    #[test]
    fn test_scattered_slice() {
        assert_eq!(TEST_SLICE.len(), 3);
        assert!(TEST_SLICE.contains(&1));
        assert!(TEST_SLICE.contains(&2));
        assert!(TEST_SLICE.contains(&3));
    }

    __slice!(@gather B pub static EMPTY_SLICE: (ScatteredSlice)<u32>;);

    #[test]
    fn test_empty_scattered_slice() {
        assert_eq!(EMPTY_SLICE.len(), 0);
        assert!(EMPTY_SLICE.is_empty());
        assert_eq!(&*EMPTY_SLICE, &[] as &[u32]);
    }
}
