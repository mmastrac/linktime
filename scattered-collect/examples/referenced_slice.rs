//! Example for `ScatteredReferencedSlice`.
use scattered_collect::{gather, referenced_slice::ScatteredReferencedSlice, scatter};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MyId(u32);

/// A scattered referenced slice of `u32`.
#[gather]
static COLLECTION: ScatteredReferencedSlice<MyId>;

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
