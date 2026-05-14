#[cfg(miri)]
pub mod miri;

#[cfg(all(not(target_family = "wasm"), not(miri), not(target_os = "windows")))]
pub mod standard;

#[cfg(all(not(miri), target_family = "wasm"))]
pub mod wasm;

#[cfg(all(not(miri), target_os = "windows"))]
pub mod windows;

// Select the appropriate bounds type for the platform.
#[cfg(all(not(miri), target_family = "wasm"))]
pub use wasm::Bounds;
#[cfg(any(miri, not(target_family = "wasm")))]
pub use PtrBounds as Bounds;

/// Constant bounds for a pointer-based section.
pub struct PtrBounds {
    pub start: *const (),
    pub end: *const (),
}

impl PtrBounds {
    pub const fn new(start: *const (), end: *const ()) -> Self {
        Self { start, end }
    }

    #[inline(always)]
    pub const fn start_ptr(&self) -> *const () {
        self.start
    }
    #[inline(always)]
    pub const fn end_ptr(&self) -> *const () {
        self.end
    }
    #[inline(always)]
    pub const fn byte_len(&self) -> usize {
        // NOTE: MSRV for non-WASM targets doesn't allow byte_offset_from,
        // so we manually implement it here.
        unsafe { (self.end.cast::<u8>()).offset_from(self.start.cast::<u8>()) as usize }
    }
}

/// A non-zero-sized type that is used to align the start and end of the
/// section.
#[repr(C)]
pub struct Alignment<T> {
    _align: [T; 0],
    _padding: u8,
}

#[allow(clippy::new_without_default)]
impl<T> Alignment<T> {
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
        {$(
            $__section:ident $__type:ident => $__prefix:tt __ $__suffix:tt;
        )*}
        AUXILIARY = $__aux_sep:literal;
        MAX_LENGTH = $__max_length:literal;
        HASH_LENGTH = $__hash_length:literal;
        VALID_SECTION_CHARS = $__valid_section_chars:literal;
    ) => {
        /// Internal macro for generating a section name.
        #[macro_export]
        #[doc(hidden)]
        macro_rules! __section_name {
            $(
                (raw $__section $__type $name:ident) => {
                    concat!(concat! $__prefix, stringify!($name), concat! $__suffix);
                };
                (raw $__section $__type $name:ident $aux:ident) => {
                    concat!(concat! $__prefix, stringify!($name), $__aux_sep, stringify!($aux), concat! $__suffix);
                };
                ($pattern:tt $__section $__type $name:ident) => {
                    $crate::__support::hash!($pattern ($__prefix) $name ($__suffix) $__hash_length $__max_length $__valid_section_chars);
                };
                ($pattern:tt $__section $__type $name:ident $aux:ident) => {
                    $crate::__support::hash!($pattern ($__prefix) ($name $__aux_sep $aux) ($__suffix) $__hash_length $__max_length $__valid_section_chars);
                };
            )*
            ($pattern:tt $unknown_section:ident $unknown_type:ident $name:ident) => {
                const _: () = {
                    compile_error!(concat!("Unknown section type: `", stringify!($unknown_section), "/", stringify!($unknown_type), "`"));
                };
            };
        }
    };
}
