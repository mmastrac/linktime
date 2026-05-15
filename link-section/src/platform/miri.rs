//! Miri: null section bounds (unsupported).

/// Miri is not currently supported.
#[doc(hidden)]
#[macro_export]
macro_rules! __get_section_miri {
    (name=$ident:ident, type=$generic_ty:ty $(, aux=$aux:ident )?) => {{
        $crate::__support::PtrBounds::new(core::ptr::null_mut(), core::ptr::null_mut())
    }};
}
