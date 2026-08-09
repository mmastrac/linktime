//! Registrations for the `copied` integration test (crate B).

use copied_types::{ComplexType, IMMUTABLE_LINK_SECTION, MUT_LINK_SECTION, OTHER_TYPE, OTHER_TYPE_2, VALUES};
use link_section::in_section;

#[in_section(VALUES)]
const _: &'static u64 = {
    static V: u64 = 20;
    &V
};

#[in_section(VALUES)]
const _: &'static u64 = {
    static V: u64 = 30;
    &V
};

#[in_section(MUT_LINK_SECTION)]
const _: ComplexType = ComplexType {
    static_string: "4",
    static_ptr: &OTHER_TYPE_2,
};

#[in_section(MUT_LINK_SECTION)]
const _: ComplexType = ComplexType {
    static_string: "5",
    static_ptr: &OTHER_TYPE,
};

#[in_section(MUT_LINK_SECTION)]
const _: ComplexType = ComplexType {
    static_string: "3",
    static_ptr: &OTHER_TYPE,
};

#[in_section(IMMUTABLE_LINK_SECTION)]
const _: ComplexType = ComplexType {
    static_string: "4",
    static_ptr: &OTHER_TYPE,
};
