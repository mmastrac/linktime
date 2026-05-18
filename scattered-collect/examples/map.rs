//! Example for `ScatteredReferencedSlice`.
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]

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

    println!("Entries:");
    for (key, value) in &MAP {
        println!(" - {}: {:?}", key, value);
    }
}
