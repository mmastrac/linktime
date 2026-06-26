use scattered_collect::{declarative::scatter, gather, map::ScatteredMap};

#[gather]
static MAP: ScatteredMap<&'static str, u32>;

scatter! {
    static BAD: (&'static str, u32) = ("a", 1);
}

fn main() {}
