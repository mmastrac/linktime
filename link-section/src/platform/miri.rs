//! Miri: null section bounds (unsupported).

/// Miri is not currently supported.
#[doc(hidden)]
#[macro_export]
macro_rules! __get_section {
    (name=$ident:ident, type=$generic_ty:ty $(, aux=$aux:ident )?) => {{
        $crate::__support::PtrBounds::new(core::ptr::null_mut(), core::ptr::null_mut())
    }};
}

// This does not currently work.
crate::__def_section_name! {
    {
        data bare =>    ("_data", "_link_section_") __ ();
        data section => ("_data", "_link_section_") __ ();
        data start =>   ("__start_", "_data", "_link_section_") __ ();
        data end =>     ("__stop_", "_data", "_link_section_") __ ();
        code bare =>    ("_text", "_link_section_") __ ();
        code section => ("_text", "_link_section_") __ ();
        code start =>   ("__start_", "_text", "_link_section_") __ ();
        code end =>     ("__stop_", "_text", "_link_section_") __ ();
    }
    AUXILIARY = "_";
    MAX_LENGTH = 64;
    HASH_LENGTH = 10;
    VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
}
