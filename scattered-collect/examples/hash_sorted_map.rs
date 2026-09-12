//! Example for `ScatteredHashSortedMap`.
use scattered_collect::{gather, hash_sorted_map::ScatteredHashSortedMap, scatter};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MyId(u32);

#[gather]
static MAP: ScatteredHashSortedMap<&'static str, MyId>;

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

    println!("Entries (link order):");
    for (key, value) in &MAP {
        println!(" - {}: {:?}", key, value);
    }

    println!("Entries (hash order):");
    for (key, value) in MAP.sorted_entries() {
        println!(" - {}: {:?}", key, value);
    }

    println!("Lookups:");
    for key in ["apple", "banana", "orange", "grape"] {
        println!(" - {key}: {:?}", MAP.find(&key));
    }
}
