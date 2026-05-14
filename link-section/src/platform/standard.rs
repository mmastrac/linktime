/// On LLVM/GCC platforms we can use orphan sections with _start and _end
/// symbols.
///
/// On Apple platforms, the linker provides a pointer to the start and end
/// of the section regardless of the section's name.
#[doc(hidden)]
#[macro_export]
macro_rules! __get_section {
    (name=$ident:ident, type=$generic_ty:ty $(, aux=$aux:ident )?) => {
        {
            // These are not valid items, but they are valid pointers.
            // We cannot safely use them - only take pointers to them.
            $crate::__support::add_section_link_attribute!(
                data start $ident $($aux)?
                #[link_name = __]
                extern "C" {
                    static __START: u8;
                }
            );
            $crate::__support::add_section_link_attribute!(
                data end $ident $($aux)?
                #[link_name = __]
                extern "C" {
                    static __END: u8;
                }
            );

            $crate::__support::PtrBounds::new(
                // TODO: black_box when hint is stable
                unsafe { &raw const __START as *const () },
                unsafe { &raw const __END as *const () },
            )
        }
    }
}

// \x01: "do not mangle" (ref https://github.com/rust-lang/rust-bindgen/issues/2935)
#[cfg(target_vendor = "apple")]
crate::__def_section_name! {
    {
        data bare =>    ("__DATA,") __ ();
        code bare =>    ("__TEXT,") __ ();
        data section => ("__DATA,") __ (",regular,no_dead_strip");
        code section => ("__TEXT,") __ (",regular,pure_instructions");
        data start =>   ("\x01section$start$__DATA$") __ ();
        data end =>     ("\x01section$end$__DATA$") __ ();
    }
    AUXILIARY = "_";
    MAX_LENGTH = 16;
    HASH_LENGTH = 6;
    VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
}

#[cfg(not(target_vendor = "apple"))]
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
