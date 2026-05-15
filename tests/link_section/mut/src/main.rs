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
}

pub fn main() {
    libc_eprintln!("MUT_LINK_SECTION: {:?}", MUT_LINK_SECTION);
    for item in MUT_LINK_SECTION {
        libc_eprintln!("item: {item}");
    }
    libc_eprintln!("AUX_MUT_LINK_SECTION: {:?}", aux_section::AUX_MUT_LINK_SECTION);
    for item in aux_section::AUX_MUT_LINK_SECTION {
        libc_eprintln!("aux item: {item}");
    }
}
