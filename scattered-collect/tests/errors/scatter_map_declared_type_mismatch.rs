//! The declared `static` type must name the map's `(Key, Value)`.
use scattered_collect::{gather, map::ScatteredMap, scatter};

#[gather]
static MAP: ScatteredMap<&'static str, u32>;

#[scatter(MAP)]
static BAD: (&'static str, u64) = ("a", 1);

fn main() {}
