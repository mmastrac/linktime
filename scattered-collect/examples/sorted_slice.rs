//! Example for `ScatteredSortedSlice`.
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]

use scattered_collect::{gather, scatter, sorted_slice::ScatteredSortedSlice};

/// A scattered sorted slice of `u32`.
#[gather]
pub static COLLECTION: ScatteredSortedSlice<u32>;

#[scatter(COLLECTION)]
const _: u32 = 1;

#[scatter(COLLECTION)]
const _: u32 = 2;

#[scatter(COLLECTION)]
const _: u32 = 3;

fn main() {
    println!("COLLECTION: {:?}", &*COLLECTION);
}
