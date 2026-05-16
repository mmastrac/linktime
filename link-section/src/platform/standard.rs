//! Non-WASM, non-Windows, non-Apple: orphan section start/end symbols.

/// On LLVM/GCC platforms we can use orphan sections with _start and _end
/// symbols.
#[doc(hidden)]
#[macro_export]
macro_rules! __get_section_standard {
    (movable, name=$ident:ident, type=$generic_ty:ty $(, aux=$aux:ident )?) => {
        {
            $crate::__support::MovableBounds::new(
                $crate::__support::PtrBounds::new(
                    $crate::__address_of_symbol!(item data start $ident $($aux)?),
                    $crate::__address_of_symbol!(item data end $ident $($aux)?),
                ),
                $crate::__support::PtrBounds::new(
                    $crate::__address_of_symbol!(backref data start $ident $($aux)?),
                    $crate::__address_of_symbol!(backref data end $ident $($aux)?),
                ),
            )
        }
    };
    ($section_type:ident, name=$ident:ident, type=$generic_ty:ty $(, aux=$aux:ident )?) => {
        {
            $crate::__support::PtrBounds::new(
                $crate::__address_of_symbol!(item data start $ident $($aux)?),
                $crate::__address_of_symbol!(item data end $ident $($aux)?),
            )
        }
    }
}

pub use crate::__get_section_standard as get_section;

crate::__def_section_name! {
    __section_name_standard,
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
    REFS = "_r_";
    MAX_LENGTH = 64;
    HASH_LENGTH = 10;
    VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
}
