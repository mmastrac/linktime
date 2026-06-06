//! Example for `ScatteredMap`.
use scattered_collect::{gather, map::ScatteredMap, scatter};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MyId(u32);

#[gather]
static MAP: ScatteredMap<&'static str, MyId>;

#[scatter(MAP)]
static APPLE: (&'static str, MyId) = ("apple", MyId(1));

#[scatter(MAP)]
static BANANA: (&'static str, MyId) = ("banana", MyId(2));

#[scatter(MAP)]
static ORANGE: (&'static str, MyId) = ("orange", MyId(3));

fn main() {
    println!("APPLE: {:?}", APPLE);
    println!("BANANA: {:?}", BANANA);
    println!("ORANGE: {:?}", ORANGE);

    assert!(ScatteredMap::offset_of(&MAP, &APPLE).is_some());
    assert!(ScatteredMap::offset_of(&MAP, &BANANA).is_some());
    assert!(ScatteredMap::offset_of(&MAP, &ORANGE).is_some());

    println!("Entries:");
    for (key, value) in &MAP {
        println!(" - {}: {:?}", key, value);
    }
}
