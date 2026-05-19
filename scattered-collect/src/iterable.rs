//! A collection of items linked at runtime into a singly-linked list in
//! constructor order.
#![doc = concat!("```rust\n", include_str!("../examples/iterable.rs"), "\n```\n")]

use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// One node in a [`ScatteredIterable`], with a `static` handle at each scatter site.
pub struct Ref<T: 'static> {
    next: AtomicPtr<Ref<T>>,
    value: T,
}

impl<T> Ref<T> {
    /// Create a new list node holding `value`.
    pub const fn new(value: T) -> Self {
        Self {
            next: AtomicPtr::new(core::ptr::null_mut()),
            value,
        }
    }
}

impl<T> ::core::ops::Deref for Ref<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> ::core::fmt::Debug for Ref<T>
where
    T: ::core::fmt::Debug,
{
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        (**self).fmt(f)
    }
}

/// Gather state for a [`ScatteredIterable`].
#[doc(hidden)]
pub struct __ScatteredIterableState<T: 'static> {
    head: AtomicPtr<Ref<T>>,
    len: AtomicUsize,
}

impl<T> __ScatteredIterableState<T> {
    #[doc(hidden)]
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(core::ptr::null_mut()),
            len: AtomicUsize::new(0),
        }
    }
}

/// Prepend `node` to `state` and increment the length.
#[doc(hidden)]
pub fn submit<T: 'static>(state: &__ScatteredIterableState<T>, node: &'static Ref<T>) {
    loop {
        let head = state.head.load(Ordering::Relaxed);
        node.next.store(head, Ordering::Relaxed);
        if state
            .head
            .compare_exchange_weak(
                head,
                core::ptr::from_ref(node).cast_mut(),
                Ordering::Release,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            state.len.fetch_add(1, Ordering::Relaxed);
            return;
        }
    }
}

/// Iterator over a [`ScatteredIterable`].
pub struct Iter<'a, T: 'static> {
    current: *const Ref<T>,
    remaining: usize,
    _marker: ::core::marker::PhantomData<&'a T>,
}

impl<'a, T: 'static> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }

        // SAFETY: Nodes are `static` and remain valid for the program lifetime.
        // Iteration happens after all priority-0 constructors have finished linking.
        let node = unsafe { &*self.current };
        let item = &node.value;
        self.current = node.next.load(Ordering::Acquire);
        self.remaining = self.remaining.saturating_sub(1);
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T: 'static> ExactSizeIterator for Iter<'_, T> {}

/// A collection of items linked at runtime into a singly-linked list.
///
/// Scatter sites register `static` [`Ref`] handles in priority-0 constructors.
/// Iteration order is the reverse of constructor registration order (each item
/// is prepended to the list).
///
/// For link-time contiguous storage in arbitrary link order, use
/// [`crate::ScatteredSlice`]. For `static` handles backed by a link section,
/// use [`crate::ScatteredReferencedSlice`]. For sorted data, use
/// [`crate::ScatteredSortedSlice`] or [`crate::ScatteredSortedReferencedSlice`].
/// For key lookup, use [`crate::ScatteredMap`].
#[doc = concat!("```rust\n", include_str!("../examples/iterable.rs"), "\n```\n")]
pub struct ScatteredIterable<T: 'static> {
    state: &'static __ScatteredIterableState<T>,
}

impl<T: 'static> ScatteredIterable<T> {
    #[doc(hidden)]
    pub const fn new(state: &'static __ScatteredIterableState<T>) -> Self {
        Self { state }
    }

    /// The number of items in the iterable.
    pub fn len(&self) -> usize {
        self.state.len.load(Ordering::Acquire)
    }

    /// True if the iterable is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: 'static> ::core::iter::IntoIterator for &'static ScatteredIterable<T> {
    type Item = &'static T;
    type IntoIter = Iter<'static, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            current: self.state.head.load(Ordering::Acquire),
            remaining: self.state.len.load(Ordering::Acquire),
            _marker: ::core::marker::PhantomData,
        }
    }
}

/// Declare a scattered iterable.
#[macro_export]
#[doc(hidden)]
macro_rules! __iterable_state {
    ($name:ident) => {
        $crate::__support::ident_concat!( () (__ $name __state__) () )
    };
}

/// Declare a scattered iterable.
#[macro_export]
#[doc(hidden)]
macro_rules! __iterable {
    (gather $vis:vis $name:ident: $ty:ty) => {
        $crate::__iterable!(@gather $vis static $name: ScatteredIterable<$ty>;);
    };
    (@gather $(#[$meta:meta])* $vis:vis static $name:ident: $collection:ident < $ty:ty >;) => {
        $crate::__support::ident_concat!((#[doc(hidden)] #[macro_export] macro_rules!) (__ $name __iterable_private_macro__) ({
            ($passthru:tt) => {
                $crate::__iterable!(@scatter $passthru);
            };
        }));

        $crate::__support::ident_concat!((#[doc(hidden)] $vis use) (__ $name __iterable_private_macro__) (as $name;));

        $crate::__support::ident_concat!((static ) (__ $name __state__) (: $crate::iterable::__ScatteredIterableState<$ty> = $crate::iterable::__ScatteredIterableState::new();));

        $(#[$meta])*
        $vis static $name: $collection<$ty> = {
            $crate::iterable::ScatteredIterable::new(&$crate::__iterable_state!($name))
        };
    };
    (scatter $collection:ident => $vis:vis $name:ident: $ty:ty = $expr:expr) => {
        $collection ! (( $collection => $vis static $name: $ty = $expr; ));
    };
    (@scatter ($collection:ident => $(#[$imeta:meta])* $vis:vis static $name:ident: $ty:ty = $expr:expr;)) => {
        $(#[$imeta])*
        $vis static $name: $crate::iterable::Ref<$ty> = $crate::iterable::Ref::new($expr);

        $crate::__support::ctor::declarative::ctor!(
            #[ctor(unsafe, anonymous, priority = 0)]
            fn __iterable_submit() {
                $crate::iterable::submit(&$crate::__iterable_state!($collection), &$name);
            }
        );
    };
}

#[cfg(all(test, not(miri)))]
mod tests {
    use crate::iterable::ScatteredIterable;

    __iterable!(gather pub TEST_ITERABLE: u32);
    __iterable!(scatter TEST_ITERABLE => pub ITERABLE_ITEM_A: u32 = 1);
    __iterable!(scatter TEST_ITERABLE => pub ITERABLE_ITEM_B: u32 = 3);
    __iterable!(scatter TEST_ITERABLE => pub ITERABLE_ITEM_C: u32 = 2);

    #[test]
    fn test_scattered_iterable() {
        assert_eq!(TEST_ITERABLE.len(), 3);
        assert!(!TEST_ITERABLE.is_empty());

        assert_eq!(*ITERABLE_ITEM_A, 1);
        assert_eq!(*ITERABLE_ITEM_B, 3);
        assert_eq!(*ITERABLE_ITEM_C, 2);

        let items: Vec<u32> = (&TEST_ITERABLE).into_iter().copied().collect();
        assert_eq!(items.len(), 3);
        assert!(items.contains(&1));
        assert!(items.contains(&2));
        assert!(items.contains(&3));
    }
}
