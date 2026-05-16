//! Apple platform: `section$start$` and `section$end$` symbols.

/// On Apple platforms, the linker provides a pointer to the start and end of
/// the section regardless of the section's name.
#[doc(hidden)]
#[macro_export]
macro_rules! __get_section_apple {
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

pub use crate::__get_section_apple as get_section;

// \x01: "do not mangle" (ref https://github.com/rust-lang/rust-bindgen/issues/2935)
crate::__def_section_name! {
    __section_name_apple,
    {
        data bare =>    ("__DATA,") __ ();
        code bare =>    ("__TEXT,") __ ();
        data section => ("__DATA,") __ (",regular,no_dead_strip");
        code section => ("__TEXT,") __ (",regular,pure_instructions");
        data start =>   ("\x01section$start$__DATA$") __ ();
        data end =>     ("\x01section$end$__DATA$") __ ();
    }
    AUXILIARY = "_";
    REFS = "_r_";
    MAX_LENGTH = 16;
    HASH_LENGTH = 6;
    VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
}

const fn find_byte(bytes: &[u8], byte: u8) -> Option<usize> {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == byte {
            return Some(i);
        }
        i += 1;
    }
    None
}

pub(crate) const fn validate_apple_section_name(name: &str) {
    let bytes = name.as_bytes();
    let comma = match find_byte(bytes, b',') {
        Some(i) => i,
        None => panic!("section name must contain a comma"),
    };
    if comma == 0 {
        panic!("section name must have a segment before the comma");
    }

    let mut i = comma + 1;
    if i >= bytes.len() {
        panic!("section name must not be empty");
    }
    let mut section_len = 0;
    while i < bytes.len() {
        if bytes[i] == b',' {
            break;
        }
        if !is_valid_section_char(bytes[i]) {
            panic!("section name contains invalid character");
        }
        section_len += 1;
        i += 1;
    }
    if section_len == 0 {
        panic!("Mach-O section name must not be empty");
    }
    if section_len > MAX_LENGTH {
        if cfg!(feature = "proc_macro") {
            // `hash!` shortens linker section names; const metadata keeps the raw name.
            return;
        }
        panic!("Mach-O section name must be 1 to 16 characters");
    }
}
