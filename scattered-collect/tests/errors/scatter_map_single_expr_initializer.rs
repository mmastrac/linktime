use scattered_collect::{gather, map::ScatteredMap, scatter};

#[gather]
static MAP: ScatteredMap<&'static str, u32>;

#[scatter(MAP)]
static BAD: (&'static str, u32) = stringify!(only_a_key);

fn main() {}
