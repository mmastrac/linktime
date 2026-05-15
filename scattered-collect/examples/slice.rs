//! Example for `ScatteredSlice`.
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]

use scattered_collect::{gather, scatter, slice::ScatteredSlice};

/// A scattered slice of `u32`.
#[gather]
pub static COLLECTION: ScatteredSlice<u32>;

#[scatter(COLLECTION)]
const _: u32 = 1;

#[scatter(COLLECTION)]
const _: u32 = 2;

#[scatter(COLLECTION)]
const _: u32 = 3;

fn main() {
    println!("COLLECTION: {:?}", &*COLLECTION);
}
