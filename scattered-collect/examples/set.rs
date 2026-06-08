//! Example for `ScatteredMap`.
use scattered_collect::{gather, set::ScatteredSet, scatter};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MyId(u32);

#[gather]
static SET: ScatteredSet<&'static str>;

#[scatter(SET)]
static APPLE: &'static str = "apple";

#[scatter(SET)]
static BANANA: &'static str = "banana";

#[scatter(SET)]
static ORANGE: &'static str = "orange";

fn main() {
    println!("APPLE: {:?}", APPLE);
    println!("BANANA: {:?}", BANANA);
    println!("ORANGE: {:?}", ORANGE);

    println!("Entries:");
    for key in &SET {
        println!(" - {:?}", key);
    }
}
