//! Example for `ScatteredIterable`.
use scattered_collect::{gather, iterable::ScatteredIterable, scatter};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MyId(u32);

/// A scattered iterable of `MyId`.
#[gather]
static COLLECTION: ScatteredIterable<MyId>;

#[scatter(COLLECTION)]
static ITEM_ONE: MyId = MyId(1);

#[scatter(COLLECTION)]
static ITEM_TWO: MyId = MyId(2);

#[scatter(COLLECTION)]
static ITEM_THREE: MyId = MyId(3);

fn main() {
    println!("COLLECTION len: {}", COLLECTION.len());
    println!("ITEM_ONE: {:?}", *ITEM_ONE);
    println!("ITEM_TWO: {:?}", *ITEM_TWO);
    println!("ITEM_THREE: {:?}", *ITEM_THREE);

    println!("Entries (head → tail):");
    for item in &COLLECTION {
        println!(" - {:?}", item);
    }
}
