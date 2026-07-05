use link_section::section;

#[derive(Copy, Clone, Debug)]
pub struct Item(pub u32);

#[section(typed)]
pub static ITEMS: link_section::TypedSection<Item>;
