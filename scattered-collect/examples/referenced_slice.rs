//! Example for `ScatteredReferencedSlice`.
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]

use scattered_collect::{gather, referenced_slice::ScatteredReferencedSlice, scatter};

/// A scattered referenced slice of `u32`.
#[gather]
static COLLECTION: ScatteredReferencedSlice<u32>;

#[scatter(COLLECTION)]
static ITEM_ONE: u32 = 1;

#[scatter(COLLECTION)]
static ITEM_TWO: u32 = 2;

#[scatter(COLLECTION)]
static ITEM_THREE: u32 = 3;

fn main() {
    println!("COLLECTION: {:?}", &*COLLECTION);
    println!("ITEM_ONE: {}", *ITEM_ONE);
}
