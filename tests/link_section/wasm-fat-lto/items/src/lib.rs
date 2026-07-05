use link_section::in_section;
use section_def::ITEMS;

// Two identical items submitted from a dependency crate.
#[in_section(ITEMS)]
const ITEM_0: section_def::Item = section_def::Item(7);

#[in_section(ITEMS)]
const ITEM_1: section_def::Item = section_def::Item(7);

/// Referenced by `app` so the linker pulls in this crate (and therefore its
/// `#[in_section]` constructors) even though nothing else uses it directly.
#[inline(never)]
pub fn touch() {
    core::hint::black_box(());
}
