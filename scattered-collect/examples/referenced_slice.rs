//! Example for `ScatteredReferencedSlice`.
use scattered_collect::{gather, scatter};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MyId(u32);

/// A scattered referenced slice of `u32`.
#[gather]
static COLLECTION: scattered_collect::referenced_slice::ScatteredReferencedSlice<MyId>;

#[scatter(COLLECTION)]
static ITEM_ONE: MyId = MyId(1);

#[scatter(COLLECTION)]
static ITEM_TWO: MyId = MyId(2);

#[scatter(COLLECTION)]
static ITEM_THREE: MyId = MyId(3);

fn main() {
    println!("COLLECTION: {:?}", &*COLLECTION);
    println!("ITEM_ONE: {:?}", ITEM_ONE);
}
