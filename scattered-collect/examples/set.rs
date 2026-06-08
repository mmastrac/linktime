//! Example for `ScatteredMap`.
use scattered_collect::{gather, scatter, set::ScatteredSet};

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
