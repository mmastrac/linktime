//! Example usage of the `link-section` crate.

use ctor::ctor;
use libc_print::*;
use link_section::{in_section, section};

#[section(mutable)]
pub static MUT_LINK_SECTION: link_section::TypedMutableSection<u32>;

#[in_section(MUT_LINK_SECTION)]
const _: u32 = 4;

#[in_section(MUT_LINK_SECTION)]
const _: u32 = 2;

#[in_section(MUT_LINK_SECTION)]
const _: u32 = 1;

#[in_section(MUT_LINK_SECTION)]
const _: u32 = 3;

#[in_section(MUT_LINK_SECTION)]
const _: u32 = 5;

#[section(movable)]
pub static MOVABLE_LINK_SECTION: link_section::TypedMovableSection<u32>;

#[in_section(MOVABLE_LINK_SECTION)]
static MOVABLE_40: u32 = 40;

#[in_section(MOVABLE_LINK_SECTION)]
static MOVABLE_20: u32 = 20;

#[in_section(MOVABLE_LINK_SECTION)]
static MOVABLE_10: u32 = 10;

#[in_section(MOVABLE_LINK_SECTION)]
static MOVABLE_30: u32 = 30;

mod aux_section {
    use ctor::ctor;
    use link_section::{in_section, section};

    #[section(mutable, aux(main = MUT_LINK_SECTION))]
    pub(crate) static AUX_MUT_LINK_SECTION: link_section::TypedMutableSection<u32>;

    #[in_section(AUX_MUT_LINK_SECTION)]
    const AUX_LINKED_U32: u32 = 1234;

    #[in_section(AUX_MUT_LINK_SECTION)]
    const AUX_LINKED_U32_2: u32 = 4321;

    #[in_section(AUX_MUT_LINK_SECTION)]
    const AUX_LINKED_U32_3: u32 = 2341;

    #[ctor(unsafe)]
    pub fn ctor() {
        let aux_section = unsafe { AUX_MUT_LINK_SECTION.as_mut_slice() };
        aux_section.sort_unstable();
    }
}

#[ctor(unsafe)]
pub fn ctor() {
    let section = unsafe { MUT_LINK_SECTION.as_mut_slice() };
    section.sort_unstable();

    {
        let movable_section = unsafe { MOVABLE_LINK_SECTION.as_mut_slice() };
        let movable_backrefs = unsafe { MOVABLE_LINK_SECTION.as_mut_backrefs() };
        assert_eq!(movable_section.len(), movable_backrefs.len());
        // Check that the backrefs are in the same order as the items.
        for (item, backref) in movable_section.iter().zip(movable_backrefs.iter()) {
            assert_eq!(backref.current_ptr(), item as *const u32);
        }
    }

    unsafe {
        MOVABLE_LINK_SECTION.sort_unstable();
    }
}

pub fn main() {
    libc_eprintln!("MUT_LINK_SECTION: {:?}", MUT_LINK_SECTION);
    for item in MUT_LINK_SECTION {
        libc_eprintln!("item: {item}");
    }
    libc_eprintln!(
        "AUX_MUT_LINK_SECTION: {:?}",
        aux_section::AUX_MUT_LINK_SECTION
    );
    for item in aux_section::AUX_MUT_LINK_SECTION {
        libc_eprintln!("aux item: {item}");
    }
    libc_eprintln!("MOVABLE_LINK_SECTION: {:?}", MOVABLE_LINK_SECTION);
    libc_eprintln!(
        "MOVABLE_BACKREFS: {}",
        unsafe { MOVABLE_LINK_SECTION.as_mut_backrefs() }.len()
    );
    for item in MOVABLE_LINK_SECTION {
        libc_eprintln!("movable item: {item}");
    }
    libc_eprintln!("MOVABLE_40: {}", *MOVABLE_40);
    libc_eprintln!("MOVABLE_20: {}", *MOVABLE_20);
    libc_eprintln!("MOVABLE_10: {}", *MOVABLE_10);
    libc_eprintln!("MOVABLE_30: {}", *MOVABLE_30);
}
