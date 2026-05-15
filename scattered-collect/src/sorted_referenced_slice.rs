//! A collection of sized items available both as a sorted slice and as stable
//! handles at each declaration site.

use core::cell::UnsafeCell;
use link_section::TypedMutableSection;

/// A collection of sized items that are available both via sorted slice and via
/// reference at the declaration site.
///
/// The gathered items are accessed via `&'static` references; the main section
/// is sorted by `T` before `main()` and ref slots are fixed up in place.
///
/// If the reference to the individual items is not required, a sorted slice may
/// be used instead.
pub struct ScatteredSortedReferencedSlice<T: Ord + 'static> {
    data: &'static TypedMutableSection<T>,
    _marker: core::marker::PhantomData<T>,
}

impl<T: Ord + 'static> ScatteredSortedReferencedSlice<T> {
    #[doc(hidden)]
    #[allow(unsafe_code)]
    pub const unsafe fn new(data: &'static TypedMutableSection<T>) -> Self {
        Self {
            data,
            _marker: core::marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
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

/// Link-section reference to an item in the slice. As the final sort order is
/// not known until after initialization, referencing an item in the slice
/// requires an indirect load.
#[repr(C)]
pub struct Ref<T> {
    tag: u32,
    ptr: UnsafeCell<*const T>,
}

impl<T> ::core::ops::Deref for Ref<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &**self.ptr.get() }
    }
}

impl<T> Ref<T> {
    pub const fn new(ptr: *const T) -> Self {
        Self {
            tag: 0,
            ptr: UnsafeCell::new(ptr),
        }
    }
}

unsafe impl<T: Sync> Sync for Ref<T> {}
unsafe impl<T: Send> Send for Ref<T> {}

/// Used by [`__sorted_referenced_slice!`] scatter arm; do not call directly.
#[macro_export]
#[doc(hidden)]
macro_rules! __sorted_referenced_slice_decl_rslot {
    (
        $collection:ident,
        $vis:vis,
        $name:ident,
        $ty:ty,
        $expr:expr
    ) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section($collection::SLOTS)]
            $vis const $name: $crate::sorted_referenced_slice::Ref<$ty> = {
                $crate::__support::link_section::declarative::in_section!(
                    #[in_section($collection::$collection)]
                    $vis const $name: $ty = $expr;
                );
                $crate::sorted_referenced_slice::Ref::new(::core::ptr::from_ref(&$name))
            };
        );
    };
}

fn co_sort_unstable_by_main<T: Ord, R>(main: &mut [T], refs: &mut [R]) {
    debug_assert_eq!(main.len(), refs.len());
    fn partition<T: Ord, R>(main: &mut [T], refs: &mut [R]) -> usize {
        let n = main.len();
        if n == 0 {
            return 0;
        }
        let pivot = n - 1;
        let mut i = 0;
        for j in 0..pivot {
            if main[j] <= main[pivot] {
                main.swap(i, j);
                refs.swap(i, j);
                i += 1;
            }
        }
        main.swap(i, pivot);
        refs.swap(i, pivot);
        i
    }

    fn recurse<T: Ord, R>(main: &mut [T], refs: &mut [R]) {
        let n = main.len();
        if n <= 1 {
            return;
        }
        let p = partition(main, refs);
        let (ml, mr) = main.split_at_mut(p);
        let (rl, rr) = refs.split_at_mut(p);
        recurse(ml, rl);
        if mr.len() > 1 {
            recurse(&mut mr[1..], &mut rr[1..]);
        }
    }

    recurse(main, refs);
}

/// Run the four-phase algorithm (sort refs by target address, co-sort main + refs,
/// repoint refs at sorted cells, restore ref slot order). `main` and `refs` must
/// have the same length; each ref must initially point at some `main` cell
/// (one-to-one). No heap allocation.
///
/// # Safety
///
/// Caller must ensure `refs` and `main` describe the same collection, with
/// unique target addresses, and that this runs exactly once before any
/// concurrent read of `refs` through [`Ref::deref`].
#[doc(hidden)]
pub unsafe fn initialize_scattered_sorted_referenced_slice<T: Ord>(
    main: &mut [T],
    refs: &mut [Ref<T>],
) {
    assert_eq!(main.len(), refs.len());
    let n = main.len();
    if n == 0 {
        return;
    }

    // Phase 1: tag by current ref index, then sort refs by target address.
    for k in 0..n {
        refs[k].tag = k as u32;
    }
    refs.sort_unstable_by_key(|ref_slot| unsafe { *ref_slot.ptr.get() });

    // Phase 2: co-sort main and refs by T.
    co_sort_unstable_by_main(main, refs);

    // Phase 3: pointers follow sorted main order.
    for i in 0..n {
        unsafe {
            *refs[i].ptr.get() = core::ptr::from_ref(&main[i]);
        }
    }

    // Phase 4: permute refs back to original slot order (tags are pre-phase-1 indices).
    for i in 0..n {
        while refs[i].tag as usize != i {
            let t = refs[i].tag as usize;
            debug_assert!(t < n);
            refs.swap(i, t);
        }
    }
}

#[macro_export]
macro_rules! __sorted_referenced_slice {
    (gather $vis:vis $name:ident: $ty:ty) => {
        #[doc(hidden)]
        $crate::__support::ident_concat!((#[macro_export] macro_rules!) (__ $name __sorted_referenced_slice_private_macro__) ({
            ($passthru:tt) => {
                $crate::__sorted_referenced_slice!(@scatter $passthru);
            };
        }));

        $crate::__support::ident_concat!((#[doc(hidden)] $vis use) (__ $name __sorted_referenced_slice_private_macro__) (as $name;));

        #[allow(unused)]
        #[allow(non_snake_case)]
        #[doc(hidden)]
        $vis mod $name {
            $crate::__support::link_section::declarative::section!(
                #[section(mutable)]
                pub static $name: $crate::__support::link_section::TypedMutableSection<$ty>;
            );

            $crate::__support::link_section::declarative::section!(
                #[section(mutable, aux(main = $name))]
                pub static SLOTS: $crate::__support::link_section::TypedMutableSection<
                    $crate::sorted_referenced_slice::Ref<$ty>
                >;
            );

            $crate::__support::ctor::declarative::ctor!(
                #[ctor(unsafe, anonymous, priority = 0)]
                fn __sorted_referenced_slice_init() {
                    unsafe {
                        let main = $name.as_mut_slice();
                        let refs = SLOTS.as_mut_slice();
                        $crate::sorted_referenced_slice::initialize_scattered_sorted_referenced_slice(
                            main,
                            refs,
                        );
                    }
                }
            );
        }

        $vis static $name: $crate::sorted_referenced_slice::ScatteredSortedReferencedSlice<$ty> = unsafe {
            $crate::sorted_referenced_slice::ScatteredSortedReferencedSlice::new(self::$name::$name.const_deref())
        };
    };
    (scatter $collection:ident => $vis:vis $name:ident: $ty:ty = $expr:expr) => {
        $collection ! (( $collection => $vis $name: $ty = $expr ));
    };
    (@scatter ($collection:ident => $vis:vis $name:ident: $ty:ty = $expr:expr)) => {
        $crate::__sorted_referenced_slice_decl_rslot!(
            $collection,
            $vis,
            $name,
            $ty,
            $expr
        );
    };
}

#[cfg(all(test, not(miri)))]
mod tests {
    __sorted_referenced_slice!(gather pub TEST_SORT_REF: u32);
    __sorted_referenced_slice!(scatter TEST_SORT_REF => pub SORT_REF_ITEM_A: u32 = 1);
    __sorted_referenced_slice!(scatter TEST_SORT_REF => pub SORT_REF_ITEM_B: u32 = 3);
    __sorted_referenced_slice!(scatter TEST_SORT_REF => pub SORT_REF_ITEM_C: u32 = 2);

    #[test]
    fn test_scattered_sorted_referenced_slice() {
        assert_eq!(TEST_SORT_REF.len(), 3);
        assert_eq!(&*TEST_SORT_REF, [1, 2, 3].as_slice());
        assert_eq!(*SORT_REF_ITEM_A, 1);
        assert_eq!(*SORT_REF_ITEM_B, 3);
        assert_eq!(*SORT_REF_ITEM_C, 2);
    }
}
