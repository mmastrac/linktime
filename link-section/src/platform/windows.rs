//! Windows: alignment markers at section bounds.

/// On Windows platforms we don't have start/end symbols, but we do have
/// section sorting so we drop a minimum-sized type with the same alignment
/// as T at the start and end of the section.
#[doc(hidden)]
#[macro_export]
macro_rules! __get_section_windows {
    (name=$ident:ident, type=$generic_ty:ty $(, aux=$aux:ident )?) => {
        {
            use $crate::__support::Alignment;
            use $crate::__support::PtrBounds;
            use $crate::__support::add_section_link_attribute;
            use core::mem;

            add_section_link_attribute!(
                data start $ident $($aux)?
                #[link_section = __]
                static __START: Alignment<$generic_ty> = Alignment::new();
            );
            add_section_link_attribute!(
                data end $ident $($aux)?
                #[link_section = __]
                static __END: Alignment<$generic_ty> = Alignment::new();
            );

            PtrBounds::new(
                unsafe {
                    let start = &raw const __START;
                    start.cast::<u8>().add(mem::size_of::<Alignment<$generic_ty>>()) as *const()
                },
                unsafe { &raw const __END as *const () },
            )
        }
    }
}

pub use crate::__get_section_windows as get_section;

crate::__def_section_name! {
    __section_name_windows,
    {
        data bare =>    (".data", "$") __ ();
        data section => (".data", "$") __ ("$b");
        data start =>   (".data", "$") __ ("$a");
        data end =>     (".data", "$") __ ("$c");
        code bare =>    (".text", "$") __ ();
        code section => (".text", "$") __ ("$b");
        code start =>   (".text", "$") __ ("$a");
        code end =>     (".text", "$") __ ("$c");
    }
    AUXILIARY = "$d$";
    MAX_LENGTH = 64;
    HASH_LENGTH = 10;
    VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
}
