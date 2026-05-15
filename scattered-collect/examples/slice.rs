use scattered_collect::{gather, scatter, slice::ScatteredSlice};

#[gather]
pub static COLLECTION: ScatteredSlice<u32>;

#[scatter(COLLECTION)]
pub const _: u32 = 1;

#[scatter(COLLECTION)]
pub const _: u32 = 2;

#[scatter(COLLECTION)]
pub const _: u32 = 3;

pub fn main() {
    println!("COLLECTION: {:?}", &*COLLECTION);
}
