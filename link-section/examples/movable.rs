//! Reference-section example for `link-section`.
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]
#![warn(missing_docs)]

use link_section::section;

/// Operations.
#[section(movable)]
static OPERATIONS: link_section::TypedMovableSection<Operation>;

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd)]
struct Operation(u32);

mod operations {
    use super::Operation;
    use link_section::in_section;

    #[in_section(super::OPERATIONS)]
    static OPERATION_1: Operation = Operation(1);

    #[in_section(super::OPERATIONS)]
    static OPERATION_3: Operation = Operation(3);

    #[in_section(super::OPERATIONS)]
    static OPERATION_6: Operation = Operation(6);

    #[in_section(super::OPERATIONS)]
    static OPERATION_7: Operation = Operation(7);

    #[in_section(super::OPERATIONS)]
    static OPERATION_8: Operation = Operation(8);

    #[in_section(super::OPERATIONS)]
    static OPERATION_2: Operation = Operation(2);

    #[in_section(super::OPERATIONS)]
    static OPERATION_4: Operation = Operation(4);

    #[in_section(super::OPERATIONS)]
    static OPERATION_5: Operation = Operation(5);

    #[in_section(super::OPERATIONS)]
    static OPERATION_9: Operation = Operation(9);

    #[in_section(super::OPERATIONS)]
    static OPERATION_10: Operation = Operation(10);
}

fn sort_operations() {
    let section = unsafe { OPERATIONS.as_mut_slice() };
    section.sort_unstable();

    let movable_section = unsafe { OPERATIONS.as_mut_slice() };
    let movable_backrefs = unsafe { OPERATIONS.as_mut_backrefs() };
    assert_eq!(movable_section.len(), movable_backrefs.len());

    for i in 0..movable_section.len() {
        for j in i + 1..movable_section.len() {
            if movable_section[i] > movable_section[j] {
                movable_section.swap(i, j);
                movable_backrefs.swap(i, j);
            }
        }
    }
}

#[allow(unsafe_code)]
fn main() {
    // This should normally be done in a `ctor`, but for this example we know
    // there are no other live threads and we do it here.
    sort_operations();

    for op in OPERATIONS {
        println!("Operation: {op:?}");
    }
}
