pub type Bucket = wide::u8x16;

pub const BUCKET_ZERO: [u8; BUCKET_SIZE] = [0; BUCKET_SIZE];
pub const BUCKET_SIZE: usize = std::mem::size_of::<Bucket>();

/// Pluggable probe strategy for the scattered map.
///
/// For [`LinearProbe`], `len` is the number of **groups** (each group is 16 slots). The iterator
/// must visit every group index `0..len` exactly once per full cycle (wrap with `% len`).
pub trait ProbeStrategy {
    fn new(len: usize, hash: u64) -> Self;
    fn next(&mut self) -> Option<usize>;
}

/// Linear probing over **groups**: start at `((hash >> 7) % len)`, then `(i+1) % len`, …
#[derive(Debug, Clone, Copy, Default)]
pub struct LinearProbe {
    len: usize,
    remaining: usize,
    pos: usize,
}

impl ProbeStrategy for LinearProbe {
    fn new(len: usize, hash: u64) -> Self {
        debug_assert!(len > 0);
        let pos = ((hash >> 7) as usize) % len;
        Self {
            len,
            remaining: len,
            pos,
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<usize> {
        if self.remaining == 0 {
            return None;
        }
        let p = self.pos;
        self.pos = (self.pos + 1) % self.len;
        self.remaining -= 1;
        Some(p)
    }
}

/// Control byte for an occupied slot: low 7 bits = `(hash & 0x7F)`, high bit
/// always set.
///
/// The high bit can be checked via SIMD bitmask.
#[inline]
pub const fn control_byte_from_hash(hash: u64) -> u8 {
    ((hash & 0x7F) as u8) | 0x80
}

#[inline]
pub fn match_mask(group: &Bucket, tag: u8) -> u32 {
    group.simd_eq(Bucket::splat(tag)).to_bitmask()
}

#[inline]
#[cfg(test)]
pub fn split_hash(index_bits: u8, hash: u64) -> (u64, usize) {
    let hash_mask: u64 = (-1_i64 as u64) << (index_bits as usize);
    let index_mask = !hash_mask;
    (hash & hash_mask, (hash & index_mask) as usize)
}
