//! Example for `ScatteredSortedSlice`.
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]

use scattered_collect::{gather, scatter, sorted_slice::ScatteredSortedSlice};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MyId(u32);

/// A scattered sorted slice of `u32`.
#[gather]
static COLLECTION: ScatteredSortedSlice<MyId>;

#[scatter(COLLECTION)]
const _: MyId = MyId(1);

#[scatter(COLLECTION)]
const _: MyId = MyId(2);

#[scatter(COLLECTION)]
const _: MyId = MyId(3);

fn main() {
    println!("COLLECTION: {:?}", &*COLLECTION);
}
