pub mod apple;
pub mod standard;
pub mod wasm;
pub mod windows;

#[cfg(target_vendor = "apple")]
pub use apple::{get_section, section_name};
#[cfg(all(
    not(target_family = "wasm"),
    not(target_os = "windows"),
    not(target_vendor = "apple")
))]
pub use standard::{get_section, section_name};
#[cfg(target_family = "wasm")]
pub use wasm::{get_section, section_name};
#[cfg(target_os = "windows")]
pub use windows::{get_section, section_name};

// Select the appropriate bounds type for the platform.
#[cfg(target_family = "wasm")]
pub use {wasm::Bounds, wasm::MovableBounds};
#[cfg(not(target_family = "wasm"))]
pub use {PtrBounds as Bounds, PtrMovableBounds as MovableBounds};

/// Rejects section names that cannot be represented on the current target.
pub const fn validate_section_name(name: &str) {
    if cfg!(target_vendor = "apple") {
        apple::validate_apple_section_name(name);
    }
    if cfg!(all(
        not(target_family = "wasm"),
        not(target_os = "windows"),
        not(target_vendor = "apple")
    )) {
        standard::is_valid_section_name(name);
    }
}

/// Constant bounds for a pointer-based section.
pub struct PtrBounds {
    /// Section start address.
    pub start: *const (),
    /// One byte past the last section byte.
    pub end: *const (),
}

impl PtrBounds {
    /// Bounds covering `[start, end)`.
    pub const fn new(start: *const (), end: *const ()) -> Self {
        Self { start, end }
    }
}

#[cfg(all(not(miri), not(target_os = "windows")))]
impl PtrBounds {
    #[inline(always)]
    /// Start as an opaque pointer.
    pub const fn start_ptr(&self) -> *const () {
        self.start
    }
    #[inline(always)]
    /// End as an opaque pointer.
    pub const fn end_ptr(&self) -> *const () {
        self.end
    }
    #[inline(always)]
    /// Length in bytes (`end - start`).
    pub const fn byte_len(&self) -> usize {
        // NOTE: MSRV for non-WASM targets doesn't allow byte_offset_from,
        // so we manually implement it here.
        unsafe { (self.end.cast::<u8>()).offset_from(self.start.cast::<u8>()) as usize }
    }
}

#[cfg(all(not(miri), target_os = "windows"))]
impl PtrBounds {
    // On Windows the bounds are the addresses of two distinct marker statics
    // (`__START` / `__END`) and the items live in *separate* statics in
    // between. A pointer that keeps its `&__START` provenance only covers the
    // `__START` allocation, so a `&[T]` spanning the items through it is UB —
    // and LLVM exploits it: it proves every element load observes `__START`'s
    // zero bytes and constant-folds `iter().copied().collect()` into a
    // zero-filled allocation, so the slice reads as all-null at runtime.
    //
    // `black_box` lowers to an inline-asm barrier with a memory clobber, which
    // yields a pointer of unknown provenance the optimizer cannot trace back to
    // `__START` — so it can no longer prove the span. An exposed-provenance
    // round-trip (`with_exposed_provenance(ptr.expose_provenance())`) is *not*
    // sufficient: it lowers to `inttoptr`/`ptrtoint`, which LLVM folds back to
    // the original pointer (and thus the original provenance) under LTO.
    // ELF/Mach-O don't need this: their bounds come from opaque linker-defined
    // `extern` symbols, which already have unknown provenance.
    #[inline(always)]
    /// Start as an opaque pointer, with provenance opacified (see the impl
    /// comment for why this is required on COFF/Windows).
    pub fn start_ptr(&self) -> *const () {
        ::core::hint::black_box(self.start)
    }
    #[inline(always)]
    /// End as an opaque pointer, with provenance opacified.
    pub fn end_ptr(&self) -> *const () {
        ::core::hint::black_box(self.end)
    }
    #[inline(always)]
    /// Length in bytes (`end - start`).
    pub fn byte_len(&self) -> usize {
        // NOTE: MSRV for non-WASM targets doesn't allow byte_offset_from,
        // so we manually implement it here.
        unsafe { (self.end.cast::<u8>()).offset_from(self.start.cast::<u8>()) as usize }
    }
}

#[cfg(miri)]
impl PtrBounds {
    /// Start as an opaque pointer.
    pub fn start_ptr(&self) -> *const () {
        self.start as usize as *const ()
    }
    /// End as an opaque pointer.
    pub fn end_ptr(&self) -> *const () {
        self.end as usize as *const ()
    }
    /// Length in bytes (`end - start`).
    pub fn byte_len(&self) -> usize {
        self.end as usize - self.start as usize
    }
}

/// Bounds for a movable section and its associated backref section.
pub struct PtrMovableBounds {
    /// Bounds for the submitted values.
    values: PtrBounds,
    /// Bounds for the submitted backrefs.
    refs: PtrBounds,
}

impl PtrMovableBounds {
    /// Create movable-section bounds.
    pub const fn new(values: PtrBounds, refs: PtrBounds) -> Self {
        Self { values, refs }
    }

    /// Start pointer for the movable item section.
    #[inline(always)]
    pub fn start_ptr(&self) -> *const () {
        self.values.start_ptr()
    }
    /// End pointer for the movable item section.
    #[inline(always)]
    pub fn end_ptr(&self) -> *const () {
        self.values.end_ptr()
    }
    /// Length in bytes of the movable item section.
    #[inline(always)]
    pub fn byte_len(&self) -> usize {
        self.values.byte_len()
    }
    /// Start pointer for the movable backref section.
    #[inline(always)]
    pub fn backrefs_start_ptr(&self) -> *const () {
        self.refs.start_ptr()
    }
    /// End pointer for the movable backref section.
    #[inline(always)]
    pub fn backrefs_end_ptr(&self) -> *const () {
        self.refs.end_ptr()
    }
    /// Length in bytes of the movable backref section.
    #[inline(always)]
    pub fn backrefs_byte_len(&self) -> usize {
        self.refs.byte_len()
    }
}

/// `UnsafeCell` that is `Sync` and `Send`.
#[repr(transparent)]
pub struct SyncUnsafeCell<T> {
    #[allow(unused)]
    cell: ::core::cell::UnsafeCell<T>,
}

impl<T> SyncUnsafeCell<T> {
    /// Create a new `SyncUnsafeCell`.
    pub const fn new(value: T) -> Self {
        Self {
            cell: ::core::cell::UnsafeCell::new(value),
        }
    }

    /// Get a raw pointer to the contained value.
    #[inline]
    pub const fn get(&self) -> *mut T {
        self.cell.get()
    }
}

unsafe impl<T> Sync for SyncUnsafeCell<T> {}
unsafe impl<T> Send for SyncUnsafeCell<T> {}

/// A non-zero-sized type that is used to align the start and end of the
/// section.
#[repr(C)]
pub struct Alignment<T> {
    _align: [T; 0],
    _padding: u8,
}

#[allow(clippy::new_without_default)]
impl<T> Alignment<T> {
    /// Zero-sized alignment anchor.
    pub const fn new() -> Self {
        Self {
            _align: [],
            _padding: 0,
        }
    }
}

/// Declares the section_name macro.
#[macro_export]
#[doc(hidden)]
macro_rules! __def_section_name {
    (
        $__name:ident,
        {$(
            $__section:ident $__type:ident => $__prefix:tt __ $__suffix:tt;
        )*}
        AUXILIARY = $__aux_sep:literal;
        REFS = $__refs_sep:literal;
        MAX_LENGTH = $__max_length:literal;
        HASH_LENGTH = $__hash_length:literal;
        VALID_SECTION_CHARS = $__valid_section_chars:literal;
    ) => {
        mod $__name {
            /// Internal macro for generating a section name.
            #[macro_export]
            #[doc(hidden)]
            macro_rules! $__name {
                $(
                    (string item $__section $__type ($name:tt () $unsafe:tt)) => {
                        $crate::__support::hash!($unsafe ($__prefix) ($name) ($__suffix) $__hash_length $__max_length $__valid_section_chars)
                    };
                    (string item $__section $__type ($aux:tt $name:tt $unsafe:tt)) => {
                        $crate::__support::hash!($unsafe ($__prefix) ($name $__aux_sep $aux) ($__suffix) $__hash_length $__max_length $__valid_section_chars)
                    };
                    (string backref $__section $__type ($name:tt () $unsafe:tt)) => {
                        $crate::__support::hash!($unsafe ($__prefix) ($name $__refs_sep) ($__suffix) $__hash_length $__max_length $__valid_section_chars)
                    };
                    (string backref $__section $__type ($aux:tt $name:tt $unsafe:tt)) => {
                        $crate::__support::hash!($unsafe ($__prefix) ($name $__aux_sep $aux $__refs_sep) ($__suffix) $__hash_length $__max_length $__valid_section_chars)
                    };
                )*
                ($pattern:tt $unknown_ref_or_item:ident $unknown_section:ident $unknown_type:ident $name:ident) => {
                    const _: () = {
                        compile_error!(concat!("Unknown section type: `", stringify!($unknown_ref_or_item), "/", stringify!($unknown_section), "/", stringify!($unknown_type), "`"));
                    }
                };
            }

            pub use $__name as section_name;
        }

        pub use $__name::section_name;

        pub(crate) const MAX_LENGTH: usize = $__max_length;
        pub(crate) const VALID_SECTION_CHARS: &[u8] = $__valid_section_chars.as_bytes();

        #[allow(unused)]
        pub(crate) const fn is_valid_section_char(b: u8) -> bool {
            let mut i = 0;
            while i < VALID_SECTION_CHARS.len() {
                if VALID_SECTION_CHARS[i] == b {
                    return true;
                }
                i += 1;
            }
            false
        }

        #[allow(unused)]
        pub(crate) const fn is_valid_section_name(name: &str) -> bool {
            let bytes = name.as_bytes();
            if bytes.is_empty() || bytes.len() > MAX_LENGTH {
                return false;
            }
            let mut i = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if !is_valid_section_char(b) {
                    return false;
                }
                i += 1;
            }
            true
        }
    };
}
