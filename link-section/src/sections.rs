use crate::__support::Bounds;

/// An untyped link section that can be used to store any type. The underlying
/// data is not enumerable.
#[repr(C)]
pub struct Section {
    name: &'static str,
    bounds: Bounds,
}

impl Section {
    #[doc(hidden)]
    pub const unsafe fn new(name: &'static str, bounds: Bounds) -> Self {
        Self { name, bounds }
    }

    /// The byte length of the section.
    #[inline]
    pub fn byte_len(&self) -> usize {
        self.bounds.byte_len()
    }

    /// The start address of the section.
    #[inline]
    pub fn start_ptr(&self) -> *const () {
        self.bounds.start_ptr()
    }
    /// The end address of the section.
    #[inline]
    pub fn end_ptr(&self) -> *const () {
        self.bounds.end_ptr()
    }

    /// Ensures that a section exists at the given path.
    #[doc(hidden)]
    pub const fn __validate<T: IsUntypedSection>(_section: &T) {}
}

impl ::core::fmt::Debug for Section {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Section")
            .field("name", &self.name)
            .field("start", &self.start_ptr())
            .field("end", &self.end_ptr())
            .field("byte_len", &self.byte_len())
            .finish()
    }
}

unsafe impl Sync for Section {}
unsafe impl Send for Section {}

// Waiting on Rust 1.78
// #[diagnostic::on_unimplemented(message = "This is not an untyped section")]
/// Marker: untyped [`Section`] handle.
pub trait IsUntypedSection {}

macro_rules! impl_bounds_fns {
    ($generic:ident) => {
        #[doc(hidden)]
        pub const unsafe fn new(name: &'static str, bounds: Bounds) -> Self {
            assert!(
                ::core::mem::size_of::<$generic>() > 0,
                "Zero-sized types are not supported"
            );
            Self {
                name,
                bounds,
                _phantom: ::core::marker::PhantomData,
            }
        }

        /// The start address of the section.
        #[inline(always)]
        pub fn start_ptr(&self) -> *const T {
            self.bounds.start_ptr() as *const T
        }

        /// The end address of the section.
        #[inline(always)]
        pub fn end_ptr(&self) -> *const T {
            self.bounds.end_ptr() as *const T
        }

        /// The stride of the typed section.
        #[inline(always)]
        pub const fn stride(&self) -> usize {
            assert!(
                ::core::mem::size_of::<T>() > 0
                    && ::core::mem::size_of::<T>() * 2 == ::core::mem::size_of::<[T; 2]>()
            );
            ::core::mem::size_of::<T>()
        }

        /// The byte length of the section.
        #[inline]
        pub fn byte_len(&self) -> usize {
            self.bounds.byte_len()
        }

        /// The number of elements in the section.
        #[inline]
        pub fn len(&self) -> usize {
            self.byte_len() / self.stride()
        }

        /// True if the section is empty.
        #[inline]
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        /// The section as a slice.
        #[inline]
        pub fn as_slice(&self) -> &[T] {
            if self.is_empty() {
                &[]
            } else {
                unsafe { ::core::slice::from_raw_parts(self.start_ptr(), self.len()) }
            }
        }
    };
}

macro_rules! impl_bounds_traits {
    ($name:ident < $generic:ident >) => {
        impl<'a, $generic> ::core::iter::IntoIterator for &'a $name<$generic> {
            type Item = &'a $generic;
            type IntoIter = ::core::slice::Iter<'a, $generic>;
            fn into_iter(self) -> Self::IntoIter {
                self.as_slice().iter()
            }
        }

        impl<T> ::core::ops::Deref for $name<$generic> {
            type Target = [$generic];
            fn deref(&self) -> &Self::Target {
                self.as_slice()
            }
        }

        impl<T> ::core::fmt::Debug for $name<$generic> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("name", &self.name)
                    .field("start", &self.start_ptr())
                    .field("end", &self.end_ptr())
                    .field("len", &self.len())
                    .field("stride", &self.stride())
                    .finish()
            }
        }

        impl<T> $crate::__support::SectionItemType for $name<$generic> {
            type Item = $generic;
        }

        impl<T> $crate::__support::SectionItemTyped<$generic> for $name<$generic> {
            type Item = $generic;
        }

        unsafe impl<$generic> Sync for $name<$generic> where $generic: Sync {}
        unsafe impl<$generic> Send for $name<$generic> where $generic: Send {}
    };
}

/// A typed link section that can be used to store any sized type. The
/// underlying data is immutable and enumerable. `static` and `const` items are
/// stored directly in the section.
///
/// `static` items are guaranteed to have a valid return from
/// [`TypedSection::offset_of`] if they are in the section.
///
/// Platform note: WASM platforms require `const` items. Use
/// [`TypedReferenceSection`] for cross-platform support for `static` items.
#[repr(C)]
pub struct TypedSection<T: 'static> {
    name: &'static str,
    bounds: Bounds,
    _phantom: ::core::marker::PhantomData<T>,
}

impl<T: 'static> TypedSection<T> {
    impl_bounds_fns!(T);

    /// The offset of the item in the section, if it is in the section.
    #[inline]
    pub fn offset_of(&self, item: &T) -> Option<usize> {
        let ptr = item as *const T;
        if ptr < self.start_ptr() || ptr >= self.end_ptr() {
            None
        } else {
            Some(unsafe { ptr.offset_from(self.start_ptr()) as usize })
        }
    }
}

impl_bounds_traits!(TypedSection<T>);

/// A mutable typed link section that can be used to store any sized type. The
/// underlying data is (unsafely) mutable and enumerable.
///
/// Only `const` items may be submitted to a [`TypedMutableSection`].
#[repr(C)]
pub struct TypedMutableSection<T: 'static> {
    name: &'static str,
    bounds: Bounds,
    _phantom: ::core::marker::PhantomData<T>,
}

impl<T: 'static> TypedMutableSection<T> {
    impl_bounds_fns!(T);

    /// The offset of the item in the section, if it is in the section.
    #[inline]
    pub fn offset_of(&self, item: &T) -> Option<usize> {
        let ptr = item as *const T;
        if ptr < self.start_ptr() || ptr >= self.end_ptr() {
            None
        } else {
            Some(unsafe { ptr.offset_from(self.start_ptr()) as usize })
        }
    }

    /// The start address of the section.
    #[inline]
    pub fn start_ptr_mut(&self) -> *mut T {
        self.bounds.start_ptr() as *mut T
    }

    /// The start address of the section.
    #[inline]
    pub fn end_ptr_mut(&self) -> *mut T {
        self.bounds.end_ptr() as *mut T
    }

    /// The section as a mutable slice.
    ///
    /// # Safety
    ///
    /// This cannot be safely used and is _absolutely unsound_ if any other
    /// slices are live.
    #[allow(clippy::mut_from_ref)]
    #[inline]
    pub unsafe fn as_mut_slice(&self) -> &mut [T] {
        if self.is_empty() {
            &mut []
        } else {
            unsafe { ::core::slice::from_raw_parts_mut(self.start_ptr() as *mut T, self.len()) }
        }
    }
}

impl_bounds_traits!(TypedMutableSection<T>);

/// A typed link section that can be used to store any sized type. The
/// underlying data is enumerable.
#[repr(C)]
pub struct TypedReferenceSection<T: 'static> {
    name: &'static str,
    bounds: Bounds,
    _phantom: ::core::marker::PhantomData<T>,
}

impl<T: 'static> TypedReferenceSection<T> {
    impl_bounds_fns!(T);

    /// The offset of the item in the section, if it is in the section.
    #[inline]
    pub fn offset_of(&self, item: &Ref<T>) -> Option<usize> {
        let ptr = item.as_ptr();
        if ptr < self.start_ptr() || ptr >= self.end_ptr() {
            None
        } else {
            Some(unsafe { ptr.offset_from(self.start_ptr()) as usize })
        }
    }
}

impl_bounds_traits!(TypedReferenceSection<T>);

/// A reference to a value in a link section. This allows platforms like WASM
/// to reference the value, even though the final location is not known until
/// after initialization.
#[repr(transparent)]
pub struct Ref<T: 'static> {
    #[cfg(target_family = "wasm")]
    ptr: ::core::cell::UnsafeCell<*const T>,
    #[cfg(not(target_family = "wasm"))]
    t: T,
}

impl<T> Ref<T> {
    #[cfg(not(target_family = "wasm"))]
    #[doc(hidden)]
    pub const fn new(t: T) -> Self {
        Self { t }
    }

    #[cfg(target_family = "wasm")]
    #[doc(hidden)]
    pub const fn new() -> Self {
        Self {
            ptr: ::core::cell::UnsafeCell::new(::core::ptr::null()),
        }
    }

    #[cfg(target_family = "wasm")]
    #[doc(hidden)]
    pub unsafe fn set(&self, ptr: *const T) {
        *self.ptr.get() = ptr;
    }

    /// Raw pointer to the value (WASM: cell; otherwise `&T` as `*const T`).
    pub fn as_ptr(&self) -> *const T {
        #[cfg(target_family = "wasm")]
        {
            unsafe { *self.ptr.get() }
        }
        #[cfg(not(target_family = "wasm"))]
        {
            &self.t as *const T
        }
    }
}

impl<T> ::core::ops::Deref for Ref<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        #[cfg(target_family = "wasm")]
        unsafe {
            ::core::ptr::read(self.ptr.get())
                .as_ref()
                .expect("Ref not initialized")
        }
        #[cfg(not(target_family = "wasm"))]
        &self.t
    }
}

unsafe impl<T> Send for Ref<T> where T: Send {}
unsafe impl<T> Sync for Ref<T> where T: Sync {}
