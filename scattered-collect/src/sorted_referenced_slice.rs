//! A collection of sized items available both as a sorted slice and as stable
//! handles at each declaration site.
//!
//! ```
//! use scattered_collect::{
//!     gather, scatter, sorted_referenced_slice::ScatteredSortedReferencedSlice,
//! };
//!
//! #[gather]
//! static HANDLERS: ScatteredSortedReferencedSlice<u32>;
//!
//! #[scatter(HANDLERS)]
//! static HIGH: u32 = 30;
//!
//! #[scatter(HANDLERS)]
//! static LOW: u32 = 10;
//!
//! fn main() {
//! # if cfg!(miri) { return; }
//!     assert_eq!(&*HANDLERS, [10, 30].as_slice());
//!     assert_eq!(*HIGH, 30);
//!     assert_eq!(*LOW, 10);
//! }
//! ```

use link_section::TypedMovableSection;

/// Link-section reference to an item in the slice. As the final sort order is
/// not known until after initialization, referencing an item in the slice
/// requires an indirect load.
pub type Ref<T> = link_section::MovableRef<T>;

/// A collection of sized items that are available both via sorted slice and via
/// reference at the declaration site.
///
/// The gathered items are accessed via `&'static` references; the main section
/// is sorted by `T` before `main()` and declaration-site handles are fixed up
/// in place.
///
/// If the reference to the individual items is not required, a sorted slice may
/// be used instead.
pub struct ScatteredSortedReferencedSlice<T: Ord + 'static> {
    data: &'static TypedMovableSection<T>,
    _marker: core::marker::PhantomData<T>,
}

impl<T: Ord + 'static> ScatteredSortedReferencedSlice<T> {
    #[doc(hidden)]
    #[allow(unsafe_code)]
    pub const unsafe fn new(data: &'static TypedMovableSection<T>) -> Self {
        Self {
            data,
            _marker: core::marker::PhantomData,
        }
    }

    /// The number of items in the sorted referenced slice.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// True if the sorted referenced slice is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The offset of the item in the slice, if it is from this slice.
    ///
    /// This is O(1), as it performs direct pointer arithmetic.
    pub fn offset_of(
        this: &Self,
        item: impl link_section::SectionItemLocation<T>,
    ) -> Option<usize> {
        TypedMovableSection::offset_of(this.data, item)
    }
}

impl<T: Ord + 'static> ::core::ops::Deref for ScatteredSortedReferencedSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.data.as_slice()
    }
}

impl<T: Ord + 'static> ::core::iter::IntoIterator for &'static ScatteredSortedReferencedSlice<T> {
    type Item = &'static T;
    type IntoIter = ::core::slice::Iter<'static, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.as_slice().iter()
    }
}

/// Used by [`__sorted_referenced_slice!`] scatter arm; do not call directly.
#[macro_export]
#[doc(hidden)]
macro_rules! __sorted_referenced_slice_decl_rslot {
    (
        $collection:ident,
        ($(#[$imeta:meta])*),
        $vis:vis,
        $name:ident,
        $ty:ty,
        $expr:expr
    ) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection, type = movable)]
            $(#[$imeta])*
            $vis static $name: $ty = $expr;
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sorted_referenced_slice {
    (gather $vis:vis $name:ident: $ty:ty) => {
        $crate::__sorted_referenced_slice!(@gather $vis static $name: ScatteredSortedReferencedSlice<$ty>;);
    };
    (@gather $(#[$meta:meta])* $vis:vis static $name:ident: $collection:ident < $ty:ty >;) => {
        $crate::__support::ident_concat!((#[doc(hidden)] #[macro_export] macro_rules!) (__ $name __sorted_referenced_slice_private_macro__) ({
            ($passthru:tt) => {
                $crate::__sorted_referenced_slice!(@scatter $passthru);
            };
        }));

        $crate::__support::ident_concat!((#[doc(hidden)] $vis use) (__ $name __sorted_referenced_slice_private_macro__) (as $name;));

        $(#[$meta])*
        $vis static $name: $collection<$ty> = const {
            $crate::__support::link_section::declarative::section!(
                #[section(movable, no_macro)]
                static $name: $crate::__support::link_section::TypedMovableSection<$ty>;
            );

            $crate::__support::ctor::declarative::ctor!(
                #[ctor(unsafe, anonymous, priority = 0)]
                fn __sorted_referenced_slice_init() {
                    unsafe {
                        $name.sort_unstable();
                    }
                }
            );

            unsafe {
                $crate::sorted_referenced_slice::ScatteredSortedReferencedSlice::new($name.const_deref())
            }
        };
    };
    (scatter $collection:ident => $vis:vis $name:ident: $ty:ty = $expr:expr) => {
        $collection ! (( $collection => $vis static $name: $ty = $expr; ));
    };
    (@scatter ($collection:ident => $(#[$imeta:meta])* $vis:vis static $name:ident: $ty:ty = $expr:expr;)) => {
        $crate::__sorted_referenced_slice_decl_rslot!(
            $collection,
            ($(#[$imeta])*),
            $vis,
            $name,
            $ty,
            $expr
        );
    };
}

#[cfg(all(test, not(miri)))]
mod tests {
    use crate::sorted_referenced_slice::{Ref, ScatteredSortedReferencedSlice};

    __sorted_referenced_slice!(gather pub TEST_SORT_REF: u32);
    __sorted_referenced_slice!(scatter TEST_SORT_REF => pub SORT_REF_ITEM_A: u32 = 1);
    __sorted_referenced_slice!(scatter TEST_SORT_REF => pub SORT_REF_ITEM_B: u32 = 3);
    __sorted_referenced_slice!(scatter TEST_SORT_REF => pub SORT_REF_ITEM_C: u32 = 2);

    #[test]
    fn test_scattered_sorted_referenced_slice() {
        assert_eq!(TEST_SORT_REF.len(), 3);
        assert_eq!(&*TEST_SORT_REF, [1, 2, 3].as_slice());
        let a: &Ref<u32> = &SORT_REF_ITEM_A;
        assert_eq!(**a, 1);
        assert_eq!(*SORT_REF_ITEM_B, 3);
        assert_eq!(*SORT_REF_ITEM_C, 2);
    }
}
