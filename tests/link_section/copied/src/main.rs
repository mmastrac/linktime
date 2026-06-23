//! Example usage of the `link-section` crate.

use link_section::{in_section, section};
use std::fmt;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ComplexType {
    static_string: &'static str,
    static_ptr: &'static OtherType,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct OtherType {
    u32: u32,
    u64: u64,
}

static OTHER_TYPE: OtherType = OtherType { u32: 1, u64: 2 };
static OTHER_TYPE_2: OtherType = OtherType { u32: 3, u64: 4 };

#[section(mutable)]
pub static MUT_LINK_SECTION: link_section::TypedMutableSection<ComplexType>;

#[in_section(MUT_LINK_SECTION)]
const _: ComplexType = ComplexType {
    static_string: "1",
    static_ptr: &OTHER_TYPE,
};

#[in_section(MUT_LINK_SECTION)]
const _: ComplexType = ComplexType {
    static_string: "2",
    static_ptr: &OTHER_TYPE,
};

#[in_section(MUT_LINK_SECTION)]
const _: ComplexType = ComplexType {
    static_string: "3",
    static_ptr: &OTHER_TYPE,
};

mod other {
    use super::*;

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
}

#[section(typed)]
pub static IMMUTABLE_LINK_SECTION: link_section::TypedSection<ComplexType>;

#[in_section(IMMUTABLE_LINK_SECTION)]
const _: ComplexType = ComplexType {
    static_string: "1",
    static_ptr: &OTHER_TYPE,
};

mod other_immutable {
    use super::*;

    #[in_section(IMMUTABLE_LINK_SECTION)]
    const _: ComplexType = ComplexType {
        static_string: "9",
        static_ptr: &OTHER_TYPE_2,
    };

    #[in_section(IMMUTABLE_LINK_SECTION)]
    const _: ComplexType = ComplexType {
        static_string: "4",
        static_ptr: &OTHER_TYPE,
    };
}

pub fn main() {
    // LLVM was optimizing these copies into memsets
    let mut copied_section = MUT_LINK_SECTION.iter().copied().collect::<Vec<_>>();
    copied_section.sort();
    eprintln!("MUTABLE: {:?}", copied_section);

    let mut copied_section = IMMUTABLE_LINK_SECTION.iter().copied().collect::<Vec<_>>();
    copied_section.sort();
    eprintln!("IMMUTABLE: {:?}", copied_section);
}
