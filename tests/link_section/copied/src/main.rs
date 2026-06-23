//! Example usage of the `link-section` crate.

use link_section::{in_section, section};

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

#[section(typed)]
static VALUES: link_section::TypedSection<&'static u64>;

// Scatter several `&'static u64` from distinct modules. The values are distinct
// and non-zero so a correct read sorts to a known order; a miscompiled read
// yields nulls.
#[in_section(VALUES)]
const _: &'static u64 = {
    static V: u64 = 50;
    &V
};
#[in_section(VALUES)]
const _: &'static u64 = {
    static V: u64 = 10;
    &V
};

mod more {
    use link_section::in_section;

    #[in_section(crate::VALUES)]
    const _: &'static u64 = {
        static V: u64 = 40;
        &V
    };
    #[in_section(crate::VALUES)]
    const _: &'static u64 = {
        static V: u64 = 20;
        &V
    };
    #[in_section(crate::VALUES)]
    const _: &'static u64 = {
        static V: u64 = 30;
        &V
    };
}

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

    let mut v: Vec<&'static u64> = VALUES.iter().copied().collect();
    println!("LEN={}", v.len());

    v.sort_unstable_by_key(|p| **p);
    let vals: Vec<u64> = v.iter().map(|p| **p).collect();
    println!("VALS={vals:?}");

    assert_eq!(v.len(), 5, "expected 5 gathered pointers");
    assert_eq!(
        vals,
        vec![10, 20, 30, 40, 50],
        "gathered pointers must read back their real values, not zeros"
    );
    assert!(
        v.iter().all(|p| **p != 0),
        "NULL pointer in gathered slice (provenance miscompile)"
    );
    println!("OK");
}
