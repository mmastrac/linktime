//! Example usage of the `link-section` crate.

use libc_print::*;
use link_section::{in_section, section};

use std::sync::atomic::{AtomicU32, Ordering};

#[allow(unused)]
#[derive(Debug)]
struct InteriorMutItem {
    value: u32,
    atomic: AtomicU32,
}

impl InteriorMutItem {
    pub const fn new(value: u32) -> Self {
        Self { value, atomic: AtomicU32::new(value) }
    }
}

#[section(typed)]
static INTERIOR_MUT_LINK_SECTION: link_section::TypedSection<InteriorMutItem>;

#[in_section(INTERIOR_MUT_LINK_SECTION)]
static A: InteriorMutItem = InteriorMutItem::new(1);

#[in_section(INTERIOR_MUT_LINK_SECTION)]
static B: InteriorMutItem = InteriorMutItem::new(2);

fn main() {
    libc_eprintln!("INTERIOR_MUT_LINK_SECTION: {:?}", INTERIOR_MUT_LINK_SECTION);
    for item in INTERIOR_MUT_LINK_SECTION {
        libc_eprintln!("item before: {item:?}");
        item.atomic.fetch_add(1, Ordering::SeqCst);
        libc_eprintln!("item after: {item:?}");
    }
}
