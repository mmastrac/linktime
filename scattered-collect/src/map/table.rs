#![allow(clippy::modulo_one, unreachable_pub)]
use crate::map::probe::{
    BUCKET_SIZE, Bucket, LinearProbe, LookupResult, ProbeStrategy, control_byte_from_hash,
    has_empty, match_mask,
};

// Cache line sizes:
//  - macOS (ARM): 128 bytes
//  - amd64: 64 bytes
//  - generic ARM: 32 bytes

// u64 hash: 8 bytes (16 x 8 = 128)
// default group: 16 bytes

/// Slots per [`MetadataStride`]. Tuned per-platform to optimize cache usage.
pub const METADATA_STRIDE: usize = 16;

pub const BUCKET_STRIDE: usize = METADATA_STRIDE / BUCKET_SIZE;
pub const HASH_STRIDE: usize = METADATA_STRIDE;

const _: () = assert!(
    METADATA_STRIDE > 0 && METADATA_STRIDE.is_multiple_of(BUCKET_SIZE),
    "METADATA_STRIDE must be a non-zero multiple of BUCKET_SIZE"
);

/// A stride of metadata. Buckets are groups, then associated hashes are
/// grouped. This is tuned per-platform to optimize cache usage.
#[repr(C)]
pub struct MetadataStride {
    pub buckets: [Bucket; BUCKET_STRIDE],
    pub hashes: [u64; HASH_STRIDE],
}

impl Default for MetadataStride {
    fn default() -> Self {
        Self::ZERO
    }
}

impl MetadataStride {
    pub const ZERO: Self = Self {
        buckets: [Bucket::splat(0); BUCKET_STRIDE],
        hashes: [0; HASH_STRIDE],
    };

    /// The maximum number of records that can be stored in this metadata stride.
    pub const CAPACITY: usize = METADATA_STRIDE;
}

/// A pre-built scattered map table. This efficiently maps a u64 to an index.
/// The index is packed into the low N bits of the u64 and effectively reduces
/// the hash space.
#[doc(hidden)]
pub struct ScatteredMapTable {
    pub metadata: &'static [MetadataStride],
    pub index_bits: u8,
}

impl ScatteredMapTable {
    // `inline(always)`: passes ato-inliner threshold otherwise.
    #[inline(always)]
    pub fn lookup(&self, h: u64) -> LookupResult {
        match self.index_bits {
            0 => LookupResult::not_found(),
            8 => lookup::<8, LinearProbe>(self, h),
            16 => lookup::<16, LinearProbe>(self, h),
            24 => lookup::<24, LinearProbe>(self, h),
            _ => unreachable!(),
        }
    }
}

#[inline]
pub fn lookup<const INDEX_BITS: u8, P: ProbeStrategy>(
    table: &ScatteredMapTable,
    h: u64,
) -> LookupResult {
    let tag = control_byte_from_hash(h);
    let hash_mask: u64 = (-1_i64 as u64) << (INDEX_BITS as usize);
    let index_mask = !hash_mask;
    let mut probe = P::new(table.metadata.len() * BUCKET_STRIDE, h);
    let masked_hash = h & hash_mask;

    while let Some(g) = probe.next() {
        let group_stride = g / BUCKET_STRIDE;
        let group_offset = g % BUCKET_STRIDE;
        let group = &table.metadata[group_stride];
        let mut bits = match_mask(&group.buckets[group_offset], tag);
        while bits != 0 {
            let lane = bits.trailing_zeros() as usize;
            let h2 = group.hashes[group_offset * BUCKET_SIZE + lane];
            if h2 & hash_mask == masked_hash {
                let idx = (h2 & index_mask) as _;
                return LookupResult::found(idx);
            }
            bits &= bits - 1;
        }
        if has_empty(&group.buckets[group_offset]) {
            return LookupResult::not_found();
        }
    }
    LookupResult::not_found()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{build::pack_hash, probe::LinearProbe};

    #[test]
    fn test_lookup_one() {
        const HASH: u64 = 0x1234567890abcdef;
        const INDEX_BITS: u8 = 16;
        static RECORDS: [MetadataStride; 1] = const {
            let mut records = [MetadataStride::ZERO];
            let mut j = 0;
            while j < records[0].buckets.len() {
                records[0].buckets[j] = Bucket::splat(0x80);
                j += 1;
            }
            let mut bucket = [0; _];
            bucket[3] = control_byte_from_hash(HASH);
            records[0].buckets[0] = Bucket::new(bucket);
            records[0].hashes[3] = pack_hash(INDEX_BITS, HASH, 15);
            records
        };

        let table = ScatteredMapTable {
            metadata: &RECORDS,
            index_bits: INDEX_BITS,
        };

        let result = lookup::<16, LinearProbe>(&table, HASH);

        assert_eq!(result.unwrap(), 15);
    }

    /// Find a record only reachable after probing past full groups. Every group is
    /// fully occupied (no empty lane), so the early-out cannot fire before the probe
    /// reaches the needle's group — modelling a valid, heavily-collided table.
    #[test]
    fn test_lookup_needle() {
        const HASH: u64 = 0xdeadbeef_deadbeef_u64;
        const INDEX_BITS: u8 = 16;
        static RECORDS: [MetadataStride; 128] = const {
            let mut records = [MetadataStride::ZERO; 128];
            let mut i = 0;
            while i < records.len() {
                let mut j = 0;
                while j < records[i].buckets.len() {
                    records[i].buckets[j] = Bucket::splat(0x80);
                    j += 1;
                }
                i += 1;
            }
            let mut bucket = [0; _];
            bucket[7] = control_byte_from_hash(HASH);
            records[99].buckets[0] = Bucket::new(bucket);
            records[99].hashes[7] = pack_hash(INDEX_BITS, HASH, 99);
            records
        };

        let table = ScatteredMapTable {
            metadata: &RECORDS,
            index_bits: INDEX_BITS,
        };

        let result = lookup::<INDEX_BITS, LinearProbe>(&table, HASH);
        assert_eq!(result.unwrap(), 99);
    }

    /// Early out test.
    #[test]
    fn test_lookup_miss_early_out() {
        const HASH: u64 = 0xdeadbeef_deadbeef_u64;
        const INDEX_BITS: u8 = 16;
        // All groups empty: the first probed group has empty lanes, so a miss must
        // return immediately.
        static RECORDS: [MetadataStride; 128] = [const { MetadataStride::ZERO }; 128];

        let table = ScatteredMapTable {
            metadata: &RECORDS,
            index_bits: INDEX_BITS,
        };

        assert!(!lookup::<INDEX_BITS, LinearProbe>(&table, HASH).is_found());
    }
}
