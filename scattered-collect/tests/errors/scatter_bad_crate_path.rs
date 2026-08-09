use scattered_collect::{gather, scatter, slice::ScatteredSlice};

#[gather]
static ITEMS: ScatteredSlice<u32>;

// `crate_path` is tolerated by the macro but the path must resolve.
#[scatter(crate_path = ::whatever, ITEMS)]
const _: u32 = 1;

fn main() {}
