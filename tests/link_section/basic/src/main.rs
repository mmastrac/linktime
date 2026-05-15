//! Example usage of the `link-section` crate.

use link_section::{in_section, section};
use libc_print::*;

/// An untyped link section with `code` linkage.
#[section(untyped)]
pub static LINK_SECTION: link_section::Section;

/// A function in the `LINK_SECTION` section.
#[in_section(LINK_SECTION)]
pub fn link_section_function() {
    libc_eprintln!("link_section_function");
}

/// A typed link section with `data` linkage.
#[section(typed)]
pub static TYPED_LINK_SECTION: link_section::TypedSection<u32>;

/// A `u32` in the `TYPED_LINK_SECTION` section.
#[in_section(TYPED_LINK_SECTION)]
pub static LINKED_U32: u32 = 1;

/// Another `u32` in the `TYPED_LINK_SECTION` section.
#[in_section(TYPED_LINK_SECTION)]
pub static LINKED_U32_2: u32 = 2;

/// Create an aux link section for `TYPED_LINK_SECTION`.
#[section(typed, aux(main = TYPED_LINK_SECTION))]
pub static AUX_LINK_SECTION: link_section::TypedSection<u32>;

/// An auxiliary section item.
#[in_section(AUX_LINK_SECTION)]
pub static AUX_LINKED_U32: u32 = 1234;

/// A function pointerarray in the `data` section.
#[section(typed)]
pub static FN_ARRAY: link_section::TypedSection<fn()>;

/// A function in the `FN_ARRAY` section.
#[in_section(FN_ARRAY)]
pub fn linked_function() {
    libc_eprintln!("linked_function");
}

/// Another function in the `FN_ARRAY` section.
#[in_section(FN_ARRAY)]
pub fn linked_function_2() {
    libc_eprintln!("linked_function_2");
}

/// Yet another function in the `FN_ARRAY` section.
#[in_section(FN_ARRAY)]
pub static OTHER_FN: fn() = link_section_function;

/// A debuggable section in the `data` section.
#[section(typed)]
pub static DEBUGGABLES: link_section::TypedSection<&'static (dyn ::core::fmt::Debug + Sync)>;

/// A debuggable in the `DEBUGGABLES` section.
#[in_section(DEBUGGABLES)]
pub static DEBUGGABLE: &'static (dyn ::core::fmt::Debug + Sync) = &1;

/// Another debuggable in the `DEBUGGABLES` section.
#[in_section(DEBUGGABLES)]
pub static DEBUGGABLE_2: &'static (dyn ::core::fmt::Debug + Sync) = &2;

/// A function pointer in the `DEBUGGABLES` section.
#[in_section(DEBUGGABLES)]
pub const DEBUGGABLE_FUNCTION: &'static (dyn ::core::fmt::Debug + Sync) = {
    struct Debuggable;
    impl ::core::fmt::Debug for Debuggable {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.write_str("debuggable_function")
        }
    }
    &Debuggable
};

#[cfg(miri)]
pub fn main() {
    libc_eprintln!("Miri is not supported for this test");
    assert_eq!(TYPED_LINK_SECTION.len(), 0);
}

#[cfg(not(miri))]
pub fn main() {
    libc_eprintln!("LINK_SECTION: {:?}", LINK_SECTION);
    link_section_function();
    libc_eprintln!("TYPED_LINK_SECTION: {:?}", TYPED_LINK_SECTION);
    libc_eprintln!("address of TYPED_LINK_SECTION[0]: {:p}", &LINKED_U32);
    libc_eprintln!("address of TYPED_LINK_SECTION[1]: {:p}", &LINKED_U32_2);
    libc_eprintln!("AUX_LINK_SECTION: {:?}", AUX_LINK_SECTION);
    for aux in AUX_LINK_SECTION {
        libc_eprintln!("aux: {:?}", aux);
    }
    assert!(TYPED_LINK_SECTION.offset_of(&LINKED_U32).is_some());
    assert!(TYPED_LINK_SECTION.offset_of(&LINKED_U32_2).is_some());
    let random_u32 = 1234567890;
    assert!(TYPED_LINK_SECTION.offset_of(&random_u32).is_none());
    libc_eprintln!("CODE_SECTION: {:?}", FN_ARRAY);
    libc_eprintln!("{:?}", FN_ARRAY.as_slice());
    for f in FN_ARRAY {
        libc_eprintln!("f: {:?}", f);
        f();
        assert!(FN_ARRAY.offset_of(f).is_some());
    }
    libc_eprintln!("DEBUGGABLES: {:?}", DEBUGGABLES.as_slice());
}
