//! A collection of items collected into a slice (link order), with each entry
//! wrapped as [`Ref`] so `static` items work on targets such as WASM.

use link_section::TypedReferenceSection;

pub use link_section::reference::Ref;

/// A collection of sized items available both as a slice over the gathered
/// section and as `static` handles at each declaration site.
///
/// The slice is in an arbitrary link order which is platform-dependent. For a
/// sorted slice without per-item handles, use [`crate::ScatteredSlice`]. For
/// sorted data with stable per-item references, use
/// [`crate::ScatteredSortedReferencedSlice`].
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
}

impl<T: 'static> ::core::ops::Deref for ScatteredReferencedSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.section.as_slice()
    }
}

/// Declare a scattered referenced slice.
#[macro_export]
macro_rules! __referenced_slice {
    (gather $vis:vis $name:ident: $ty:ty) => {
        #[doc(hidden)]
        $crate::__support::ident_concat!((#[macro_export] macro_rules!) (__ $name __referenced_slice_private_macro__) ({
            ($passthru:tt) => {
                $crate::__referenced_slice!(@scatter $passthru);
            };
        }));

        $crate::__support::ident_concat!((#[doc(hidden)] $vis use) (__ $name __referenced_slice_private_macro__) (as $name;));

        #[allow(unused)]
        #[allow(non_snake_case)]
        #[doc(hidden)]
        $vis mod $name {
            $crate::__support::link_section::declarative::section!(
                #[section(reference)]
                pub static $name: $crate::__support::link_section::TypedReferenceSection<$ty>;
            );
        }

        $vis static $name: $crate::referenced_slice::ScatteredReferencedSlice<$ty> = {
            unsafe {
                $crate::referenced_slice::ScatteredReferencedSlice::new(
                    self::$name::$name.const_deref(),
                )
            }
        };
    };
    (scatter $collection:ident => $vis:vis $name:ident: $ty:ty = $expr:expr) => {
        $collection ! (( $collection => $vis $name: $ty = $expr ));
    };
    (@scatter ($collection:ident => $vis:vis $name:ident: $ty:ty = $expr:expr)) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section($collection::$collection)]
            $vis static $name: $ty = $expr;
        );
    };
}

#[cfg(all(test, not(miri)))]
mod tests {
    __referenced_slice!(gather pub TEST_REF_SLICE: u32);
    __referenced_slice!(scatter TEST_REF_SLICE => pub REF_SLICE_ITEM_A: u32 = 1);
    __referenced_slice!(scatter TEST_REF_SLICE => pub REF_SLICE_ITEM_B: u32 = 3);
    __referenced_slice!(scatter TEST_REF_SLICE => pub REF_SLICE_ITEM_C: u32 = 2);

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
}
