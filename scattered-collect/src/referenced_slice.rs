//! A collection of items gathered into a slice (link order), with each entry
//! wrapped as [`Ref`] so `static` items work on targets such as WASM.
#![doc = concat!("```rust\n", include_str!("../examples/referenced_slice.rs"), "\n```\n")]

use link_section::TypedReferenceSection;

pub use link_section::Ref;

/// A collection of sized items available both as a slice over the gathered
/// section and as `static` handles at each declaration site.
///
/// The slice is in an arbitrary link order which is platform-dependent. For a
/// sorted slice without per-item handles, use [`crate::ScatteredSortedSlice`]. For
/// arbitrary link order without per-item handles, use [`crate::ScatteredSlice`]. For
/// sorted data with stable per-item references, use
/// [`crate::ScatteredSortedReferencedSlice`]. For key lookup, use [`crate::ScatteredMap`].
/// ```rust
#[doc = include_str!("../examples/referenced_slice.rs")]
/// ```
pub struct ScatteredReferencedSlice<T: 'static> {
    section: &'static TypedReferenceSection<T>,
}

impl<T: 'static> ScatteredReferencedSlice<T> {
    #[doc(hidden)]
    #[allow(unsafe_code)]
    pub const unsafe fn new(section: &'static TypedReferenceSection<T>) -> Self {
        Self { section }
    }

    /// The number of items in the slice.
    pub fn len(&self) -> usize {
        self.section.len()
    }

    /// True if the slice is empty.
    pub fn is_empty(&self) -> bool {
        self.section.is_empty()
    }

    /// The offset of the item in the slice, if it is from this slice.
    ///
    /// This is O(1), as it performs direct pointer arithmetic.
    pub fn offset_of(
        this: &Self,
        item: impl link_section::SectionItemLocation<T>,
    ) -> Option<usize> {
        TypedReferenceSection::offset_of(this.section, item)
    }
}

impl<T: 'static> ::core::ops::Deref for ScatteredReferencedSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.section.as_slice()
    }
}

/// Declare a scattered referenced slice.
#[macro_export]
#[doc(hidden)]
macro_rules! __referenced_slice {
    (@gather $unique:ident $(#[$meta:meta])* $vis:vis static $name:ident: ($($collection:tt)*) < $ty:ty >;) => {
        $(#[$meta])*
        $vis static $name: $($collection)* <$ty> = {
            $crate::__support::link_section::declarative::section!(
                #[section(unsafe, type = reference, name = $name :: $unique)]
                static $name: $crate::__support::link_section::TypedReferenceSection<$ty>;
            );

            unsafe {
                $crate::referenced_slice::ScatteredReferencedSlice::new(
                    $name.const_deref(),
                )
            }
        };
    };
    (@scatter [$collection_name:ident :: $unique:ident] $types:tt ([$($meta:tt)*] => $(#[$imeta:meta])* $vis:vis static $name:ident: $ty:ty = $expr:expr;)) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection_name :: $unique, type = reference)]
            $vis static $name: $ty = $expr;
        );
    };
}

#[cfg(all(test, not(miri)))]
mod tests {
    use crate::referenced_slice::ScatteredReferencedSlice;

    __referenced_slice!(@gather A pub static TEST_REF_SLICE: (ScatteredReferencedSlice)<u32>;);
    __referenced_slice!(@scatter [TEST_REF_SLICE::A] [u32] ([TEST_REF_SLICE] => pub static REF_SLICE_ITEM_A: u32 = 1;));
    __referenced_slice!(@scatter [TEST_REF_SLICE::A] [u32] ([TEST_REF_SLICE] => pub static REF_SLICE_ITEM_B: u32 = 3;));
    __referenced_slice!(@scatter [TEST_REF_SLICE::A] [u32] ([TEST_REF_SLICE] => pub static REF_SLICE_ITEM_C: u32 = 2;));

    #[test]
    fn test_scattered_referenced_slice() {
        assert_eq!(TEST_REF_SLICE.len(), 3);
        assert!(TEST_REF_SLICE.contains(&1));
        assert!(TEST_REF_SLICE.contains(&2));
        assert!(TEST_REF_SLICE.contains(&3));

        assert_eq!(*REF_SLICE_ITEM_A, 1);
        assert_eq!(*REF_SLICE_ITEM_B, 3);
        assert_eq!(*REF_SLICE_ITEM_C, 2);
    }

    __referenced_slice!(@gather B pub static EMPTY_REF_SLICE: (ScatteredReferencedSlice)<u32>;);

    #[test]
    fn test_empty_scattered_referenced_slice() {
        assert_eq!(EMPTY_REF_SLICE.len(), 0);
        assert!(EMPTY_REF_SLICE.is_empty());
        assert_eq!(&*EMPTY_REF_SLICE, &[] as &[u32]);
    }
}
