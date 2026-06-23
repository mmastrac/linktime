//! Example usage of the `link-section` crate.

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

#[section(typed)]
pub static IMMUTABLE_LINK_SECTION: link_section::TypedSection<u32>;

#[in_section(IMMUTABLE_LINK_SECTION)]
const _: u32 = 4;

#[in_section(IMMUTABLE_LINK_SECTION)]
const _: u32 = 2;

#[in_section(IMMUTABLE_LINK_SECTION)]
const _: u32 = 1;

pub fn main() {
    // LLVM was optimizing these copies into memsets
    let mut copied_section = MUT_LINK_SECTION.iter().copied().collect::<Vec<_>>();
    copied_section.sort();
    eprintln!("MUTABLE: {:?}", copied_section);

    let mut copied_section = IMMUTABLE_LINK_SECTION.iter().copied().collect::<Vec<_>>();
    copied_section.sort();
    eprintln!("IMMUTABLE: {:?}", copied_section);
}
