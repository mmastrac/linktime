//! Shared types and link sections for the `copied` integration test.

use link_section::section;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ComplexType {
    pub static_string: &'static str,
    pub static_ptr: &'static OtherType,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct OtherType {
    pub u32: u32,
    pub u64: u64,
}

pub static OTHER_TYPE: OtherType = OtherType { u32: 1, u64: 2 };
pub static OTHER_TYPE_2: OtherType = OtherType { u32: 3, u64: 4 };

#[section(typed)]
pub static VALUES: link_section::TypedSection<&'static u64>;

#[section(mutable)]
pub static MUT_LINK_SECTION: link_section::TypedMutableSection<ComplexType>;

#[section(typed)]
pub static IMMUTABLE_LINK_SECTION: link_section::TypedSection<ComplexType>;
