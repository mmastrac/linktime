//! A collection of items available via sorted slice.
//!
#![doc = concat!("```rust\n", include_str!("../examples/sorted_slice.rs"), "\n```\n")]

use link_section::TypedMutableSection;

/// A collection of sized items available via sorted slice.
///
/// The main section is sorted by `T` in a priority-0 constructor before `main()`.
///
/// For arbitrary link order without per-item handles, use [`crate::ScatteredSlice`]. For
/// arbitrary order with `static` handles, use [`crate::ScatteredReferencedSlice`]. For
/// sorted data with stable per-item references, use
/// [`crate::ScatteredSortedReferencedSlice`]. For key lookup, use [`crate::ScatteredMap`].
#[doc = concat!("```rust\n", include_str!("../examples/sorted_slice.rs"), "\n```\n")]
pub struct ScatteredSortedSlice<T: Ord + 'static> {
    data: &'static TypedMutableSection<T>,
    _marker: core::marker::PhantomData<T>,
}

impl<T: Ord + 'static> ScatteredSortedSlice<T> {
    #[doc(hidden)]
    #[allow(unsafe_code)]
    pub const unsafe fn new(data: &'static TypedMutableSection<T>) -> Self {
        Self {
            data,
            _marker: core::marker::PhantomData,
        }
    }

    /// The number of items in the sorted slice.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// True if the sorted slice is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<T: Ord + 'static> ::core::ops::Deref for ScatteredSortedSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.data.as_slice()
    }
}

impl<T: Ord + 'static> ::core::iter::IntoIterator for &'static ScatteredSortedSlice<T> {
    type Item = &'static T;
    type IntoIter = ::core::slice::Iter<'static, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.as_slice().iter()
    }
}

#[doc(hidden)]
pub fn initialize_scattered_sorted_slice<T: Ord>(main: &mut [T]) {
    let n = main.len();
    if n == 0 {
        return;
    }

    main.sort_unstable();
}

/// Declare a scattered sorted slice.
#[doc(hidden)]
#[macro_export]
macro_rules! __sorted_slice {
    (@gather $unique:ident $(#[$meta:meta])* $vis:vis static $name:ident: ($($collection:tt)*) < $ty:ty >;) => {
        $(#[$meta])*
        $vis static $name: $($collection)* <$ty> = {
            $crate::__support::link_section::declarative::section!(
                #[section(unsafe, type = mutable, name = $name :: $unique)]
                static $name: $crate::__support::link_section::TypedMutableSection<$ty>;
            );

            $crate::__support::ctor::declarative::ctor!(
                // Run as soon as possible, before any other constructors.
                #[ctor(unsafe, anonymous, priority = 0)]
                fn initialize_scattered_sorted_slice() {
                    unsafe { $crate::sorted_slice::initialize_scattered_sorted_slice($name.as_mut_slice()) };
                }
            );

            unsafe { $crate::sorted_slice::ScatteredSortedSlice::new(
                $name.const_deref()
            ) }
        };
    };

    (@scatter [$collection_name:ident :: $unique:ident] ([$($meta:tt)*] => $(#[$imeta:meta])* $vis:vis $type:ident $name:tt: $ty:ty = $expr:expr;)) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection_name :: $unique, type = mutable)]
            $(#[$imeta])*
            $vis $type $name: $ty = $expr;
        );
    };
}

#[cfg(all(test, not(miri)))]
mod tests {
    pub use super::ScatteredSortedSlice;

    __sorted_slice!(@gather A pub static TEST_SORTED_SLICE: (ScatteredSortedSlice)<u32>;);
    __sorted_slice!(@scatter [TEST_SORTED_SLICE::A] ([TEST_SORTED_SLICE] => const _: u32 = 6;));
    __sorted_slice!(@scatter [TEST_SORTED_SLICE::A] ([TEST_SORTED_SLICE] => const _: u32 = 3;));
    __sorted_slice!(@scatter [TEST_SORTED_SLICE::A] ([TEST_SORTED_SLICE] => const _: u32 = 2;));
    __sorted_slice!(@scatter [TEST_SORTED_SLICE::A] ([TEST_SORTED_SLICE] => const _: u32 = 4;));
    __sorted_slice!(@scatter [TEST_SORTED_SLICE::A] ([TEST_SORTED_SLICE] => const _: u32 = 5;));
    __sorted_slice!(@scatter [TEST_SORTED_SLICE::A] ([TEST_SORTED_SLICE] => const _: u32 = 1;));

    #[test]
    fn test_scattered_sorted_slice() {
        assert_eq!(TEST_SORTED_SLICE.len(), 6);
        assert_eq!(&*TEST_SORTED_SLICE, [1, 2, 3, 4, 5, 6].as_slice());
        for item in &TEST_SORTED_SLICE {
            println!("item: {}", item);
        }
    }

    __sorted_slice!(@gather B pub static EMPTY_SORTED_SLICE: (ScatteredSortedSlice)<u32>;);

    #[test]
    fn test_empty_scattered_sorted_slice() {
        assert_eq!(EMPTY_SORTED_SLICE.len(), 0);
        assert!(EMPTY_SORTED_SLICE.is_empty());
        assert_eq!(&*EMPTY_SORTED_SLICE, &[] as &[u32]);
    }
}
