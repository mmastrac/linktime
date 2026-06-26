//! A collection of items linked at runtime into a singly-linked list in
//! constructor order.
#![doc = concat!("```rust\n", include_str!("../examples/iterable.rs"), "\n```\n")]

use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// Index stored in [`Ref::index`] before [`submit`] runs.
const UNSET_INDEX: usize = 0;

/// One node in a [`ScatteredIterable`], with a `static` handle at each scatter site.
pub struct Ref<T: 'static> {
    next: AtomicPtr<Ref<T>>,
    index: AtomicUsize,
    value: T,
}

impl<T> Ref<T> {
    /// Create a new list node holding `value`.
    pub const fn new(value: T) -> Self {
        Self {
            next: AtomicPtr::new(core::ptr::null_mut()),
            index: AtomicUsize::new(UNSET_INDEX),
            value,
        }
    }

    /// The offset of this node in its collection's submission order.
    pub fn offset_of(this: &Self) -> usize {
        this.loaded_index().saturating_sub(1)
    }

    #[inline]
    fn loaded_index(&self) -> usize {
        let index = self.index.load(Ordering::Acquire);
        if index == UNSET_INDEX {
            panic!("node index is unset; was this `Ref` submitted to its collection?");
        }
        index
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
            let index = state.len.fetch_add(1, Ordering::Relaxed);
            node.index.store(index + 1, Ordering::Release);
            return;
        }
    }
}

#[inline]
#[allow(unsafe_code)]
fn remaining_from_node<T>(node: *const Ref<T>) -> usize {
    if node.is_null() {
        return 0;
    }
    // SAFETY: `node` is a live `static` [`Ref`] for the program lifetime.
    unsafe { (*node).loaded_index() }
}

/// Iterator over a [`ScatteredIterable`].
pub struct Iter<'a, T: 'static> {
    current: *const Ref<T>,
    remaining: usize,
    _marker: ::core::marker::PhantomData<&'a T>,
}

impl<'a, T: 'static> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }

        // SAFETY: Nodes are `static` and remain valid for the program lifetime.
        let node = unsafe { &*self.current };
        let _ = node.loaded_index();
        let item = &node.value;
        self.current = node.next.load(Ordering::Acquire);
        self.remaining = remaining_from_node(self.current);
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T: 'static> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.remaining
    }
}

/// A collection of items linked at runtime into a singly-linked list.
///
/// Scatter sites register `static` [`Ref`] handles in priority-0 constructors.
/// Iteration order is the reverse of constructor registration order (each item
/// is prepended to the list).
///
/// ## Thread safety
///
/// This collection is thread-safe: access before initialization completes will
/// yield fewer items, but will not panic or cause undefined behavior. The count
/// returned by [`ScatteredIterable::len`] may be stale while constructors are still running; the
/// [`ExactSizeIterator::len`] on [`Iter`] is derived from each node's [`Ref::offset_of`]
/// index and stays consistent for the suffix being iterated.
///
/// For link-time contiguous storage in arbitrary link order, use
/// [`crate::ScatteredSlice`]. For `static` handles backed by a link section,
/// use [`crate::ScatteredReferencedSlice`]. For sorted data, use
/// [`crate::ScatteredSortedSlice`] or
/// [`crate::ScatteredSortedReferencedSlice`]. For key lookup, use
/// [`crate::ScatteredMap`].
#[doc = concat!("```rust\n", include_str!("../examples/iterable.rs"), "\n```\n")]
pub struct ScatteredIterable<T: 'static> {
    state: &'static __ScatteredIterableState<T>,
}

impl<T: 'static> ScatteredIterable<T> {
    #[doc(hidden)]
    pub const fn new(state: &'static __ScatteredIterableState<T>) -> Self {
        Self { state }
    }

    /// Register a scattered node with this iterable.
    #[doc(hidden)]
    pub fn __submit(&self, node: &'static Ref<T>) {
        submit(self.state, node);
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
        let current = self.state.head.load(Ordering::Acquire);
        Iter {
            remaining: remaining_from_node(current),
            current,
            _marker: ::core::marker::PhantomData,
        }
    }
}

/// Declare a scattered iterable.
#[macro_export]
#[doc(hidden)]
macro_rules! __iterable_state {
    ($name:ident) => {
        $crate::__support::combine!(output=ident prefix=() input=(__ $name __ state __) suffix=())
    };
}

/// Declare a scattered iterable.
#[macro_export]
#[doc(hidden)]
macro_rules! __iterable {
    (@gather $unique:ident $(#[$meta:meta])* $vis:vis static $name:ident: ($($collection:tt)*) < $ty:ty >;) => {
        $crate::__support::combine!(
            output=ident
            prefix=(static )
            input=(__ $name __ state __)
        suffix=(: $crate::iterable::__ScatteredIterableState<$ty> = $crate::iterable::__ScatteredIterableState::new();));

        $(#[$meta])*
        $vis static $name: $($collection)* <$ty> = {
            $crate::iterable::ScatteredIterable::new(&$crate::__iterable_state!($name))
        };
    };
    (@scatter [$collection_name:ident :: $unique:ident] ([$($meta:tt)*] => $(#[$imeta:meta])* $vis:vis static $name:ident: $ty:ty = $expr:expr;)) => {
        $(#[$imeta])*
        $vis static $name: $crate::iterable::Ref<$ty> = $crate::iterable::Ref::new($expr);

        $crate::__support::ctor::declarative::ctor!(
            #[ctor(unsafe, anonymous, priority = 0)]
            fn __iterable_submit() {
                $($meta)*.__submit(&$name);
            }
        );
    };
}

#[cfg(all(test, not(miri)))]
mod tests {
    use crate::iterable::{Ref, ScatteredIterable};

    __iterable!(@gather pub static TEST_ITERABLE: (ScatteredIterable)<u32>;);
    __iterable!(@scatter [TEST_ITERABLE :: A] ([TEST_ITERABLE] => pub static ITERABLE_ITEM_A: u32 = 1;));
    __iterable!(@scatter [TEST_ITERABLE :: A] ([TEST_ITERABLE] => pub static ITERABLE_ITEM_B: u32 = 3;));
    __iterable!(@scatter [TEST_ITERABLE :: A] ([TEST_ITERABLE] => pub static ITERABLE_ITEM_C: u32 = 2;));

    fn offset_for_value(value: u32) -> usize {
        match value {
            1 => Ref::offset_of(&ITERABLE_ITEM_A),
            3 => Ref::offset_of(&ITERABLE_ITEM_B),
            2 => Ref::offset_of(&ITERABLE_ITEM_C),
            _ => panic!("unexpected value: {value}"),
        }
    }

    #[test]
    fn test_scattered_iterable() {
        assert_eq!(TEST_ITERABLE.len(), 3);
        assert!(!TEST_ITERABLE.is_empty());

        assert_eq!(*ITERABLE_ITEM_A, 1);
        assert_eq!(*ITERABLE_ITEM_B, 3);
        assert_eq!(*ITERABLE_ITEM_C, 2);

        let mut scatter_offsets = [
            Ref::offset_of(&ITERABLE_ITEM_A),
            Ref::offset_of(&ITERABLE_ITEM_B),
            Ref::offset_of(&ITERABLE_ITEM_C),
        ];
        scatter_offsets.sort();
        assert_eq!(scatter_offsets, [0, 1, 2]);

        let total = TEST_ITERABLE.len();
        let mut it = (&TEST_ITERABLE).into_iter();
        assert_eq!(it.len(), total);

        let mut collected_offsets = Vec::new();
        for step in 0..total {
            let len_before = it.len();
            let value = it.next().copied().expect("item");
            let offset = offset_for_value(value);

            assert_eq!(
                len_before,
                offset + 1,
                "ExactSizeIterator::len before consuming offset {offset}"
            );
            assert_eq!(it.len(), total - step - 1);
            collected_offsets.push(offset);
        }

        assert_eq!(it.len(), 0);
        assert_eq!(it.next(), None);

        collected_offsets.sort();
        assert_eq!(collected_offsets, scatter_offsets);

        let items: Vec<u32> = (&TEST_ITERABLE).into_iter().copied().collect();
        assert_eq!(items.len(), total);
        assert!(items.contains(&1));
        assert!(items.contains(&2));
        assert!(items.contains(&3));
    }

    __iterable!(@gather pub static EMPTY_ITERABLE: (ScatteredIterable)<u32>;);

    #[test]
    fn test_empty_scattered_iterable() {
        assert_eq!(EMPTY_ITERABLE.len(), 0);
        assert!(EMPTY_ITERABLE.is_empty());
        assert_eq!((&EMPTY_ITERABLE).into_iter().count(), 0);
    }
}
