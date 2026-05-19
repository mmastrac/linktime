//! Example for `ScatteredSlice`.
use scattered_collect::{gather, scatter, slice::ScatteredSlice};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MyId(u32);

/// A scattered slice of `u32`.
#[gather]
static COLLECTION: ScatteredSlice<MyId>;

#[scatter(COLLECTION)]
const _: MyId = MyId(1);

#[scatter(COLLECTION)]
const _: MyId = MyId(2);

#[scatter(COLLECTION)]
const _: MyId = MyId(3);

fn main() {
    println!("COLLECTION: {:?}", &*COLLECTION);
}
