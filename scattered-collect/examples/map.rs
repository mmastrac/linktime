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

// Map keys and values can be `const`-computed!
#[scatter(MAP)]
static ORANGE: (&'static str, MyId) = const {
    let orange = "orange";
    let id = orange.len() / 2;
    (orange, MyId(id as _))
};

fn main() {
    println!("APPLE: {:?}", APPLE);
    println!("BANANA: {:?}", BANANA);
    println!("ORANGE: {:?}", ORANGE);

    println!("Entries:");
    for (key, value) in &MAP {
        println!(" - {}: {:?}", key, value);
    }
}
