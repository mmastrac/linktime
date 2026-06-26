use scattered_collect::{gather, map::ScatteredMap, scatter};

#[gather]
static MAP: ScatteredMap<&'static str, u32>;

#[scatter]
static BAD: (&'static str, u32) = ("a", 1);

fn main() {}
