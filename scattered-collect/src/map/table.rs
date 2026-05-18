#![allow(clippy::modulo_one, unreachable_pub)]

use crate::map::probe::{BUCKET_SIZE, Bucket, ProbeStrategy, control_byte_from_hash, match_mask};

// Cache line sizes:
//  - macOS (ARM): 128 bytes
//  - amd64: 64 bytes
//  - generic ARM: 32 bytes

// u64 hash: 8 bytes (16 x 8 = 128)
// default group: 16 bytes

pub const BUCKET_STRIDE: usize = 1;
pub const HASH_STRIDE: usize = BUCKET_STRIDE * BUCKET_SIZE;

/// A stride of metadata. Buckets are groups, then associated hashes are
/// grouped. This is tuned per-platform to optimize cache usage.
#[repr(C)]
#[derive(Default)]
pub struct MetadataStride {
    pub buckets: [Bucket; BUCKET_STRIDE],
    pub hashes: [u64; HASH_STRIDE],
}

impl MetadataStride {
    pub const ZERO: Self = Self {
        buckets: [Bucket::splat(0)],
        hashes: [0; HASH_STRIDE],
    };

    /// The maximum number of records that can be stored in this metadata stride.
    pub const CAPACITY: usize = HASH_STRIDE;
}

/// A pre-built scattered map table. This efficiently maps a u64 to an index.
/// The index is packed into the low N bits of the u64 and effectively reduces
/// the hash space.
///
/// Since there is a (miniscule) chance of hash collision, a lookup will return
/// a continuation key to allow for future probing.
#[doc(hidden)]
pub struct ScatteredMapTable {
    pub metadata: &'static [MetadataStride],
    pub lookup_fn: fn(&ScatteredMapTable, h: u64) -> Option<u64>,
    pub index_bits: u8,
}

pub fn lookup<const INDEX_BITS: u8, P: ProbeStrategy>(
    table: &ScatteredMapTable,
    h: u64,
) -> Option<u64> {
    let tag = control_byte_from_hash(h);
    let hash_mask: u64 = (-1_i64 as u64) << (INDEX_BITS as usize);
    let index_mask = !hash_mask;
    let mut probe = P::new(table.metadata.len(), h);
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
                return Some(h2 & index_mask);
            }
            bits &= bits - 1;
        }
    }
    None
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

            let mut bucket = [0; _];
            bucket[3] = control_byte_from_hash(HASH);
            records[0].buckets[0] = Bucket::new(bucket);
            records[0].hashes[3] = pack_hash(INDEX_BITS, HASH, 15);
            records
        };

        let table = ScatteredMapTable {
            metadata: &RECORDS,
            lookup_fn: lookup::<INDEX_BITS, LinearProbe>,
            index_bits: INDEX_BITS,
        };

        let result = lookup::<16, LinearProbe>(&table, HASH);

        assert_eq!(result, Some(15));
    }

    /// Find a single record somewhere in the table.
    #[test]
    fn test_lookup_needle() {
        const HASH: u64 = 0xdeadbeef_deadbeef_u64;
        const INDEX_BITS: u8 = 16;
        static RECORDS: [MetadataStride; 128] = const {
            let mut records = [MetadataStride::ZERO; 128];

            let mut bucket = [0; _];
            bucket[7] = control_byte_from_hash(HASH);
            records[99].buckets[0] = Bucket::new(bucket);
            records[99].hashes[7] = pack_hash(INDEX_BITS, HASH, 99);
            records
        };

        let table = ScatteredMapTable {
            metadata: &RECORDS,
            lookup_fn: lookup::<16, LinearProbe>,
            index_bits: INDEX_BITS,
        };

        let result = lookup::<INDEX_BITS, LinearProbe>(&table, HASH);
        assert_eq!(result, Some(99));
    }
}
