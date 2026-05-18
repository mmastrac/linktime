use crate::map::MapRecord;
use crate::map::probe::{BUCKET_SIZE, Bucket, LinearProbe, ProbeStrategy, control_byte_from_hash};
use crate::map::table::{BUCKET_STRIDE, MetadataStride, ScatteredMapTable, lookup};

pub const SAFE_CAPACITY: f32 = 0.70;
pub const BASE_CAPACITY: usize = 512;
pub const PER_ITEM_CAPACITY: usize = 14;

pub const fn safe_record_count_for_capacity(capacity: usize) -> usize {
    (capacity * 100 / ((SAFE_CAPACITY * 100.0) as usize)) / MetadataStride::CAPACITY as usize + 2
}

pub const fn pack_hash(index_bits: u8, hash: u64, index: usize) -> u64 {
    let hash_mask: u64 = (-1_i64 as u64) << (index_bits as usize);
    let index_mask = !hash_mask;
    hash & hash_mask | (index as u64) & index_mask
}

/// First lane index in `buckets` whose byte is `0` (empty), using SIMD.
#[inline]
pub fn first_empty_lane(buckets: &Bucket) -> Option<usize> {
    let mask = buckets.simd_eq(Bucket::splat(0)).to_bitmask();
    if mask == 0 {
        None
    } else {
        Some(mask.trailing_zeros() as usize)
    }
}

/// Initialize a scattered map table from a slice of records.
pub fn initialize_scattered_map<K, V>(
    records: &[MapRecord<K, V>],
    refs: &'static mut [u8],
) -> ScatteredMapTable {
    let n = records.len();
    if n == 0 {
        return ScatteredMapTable {
            metadata: &[],
            lookup_fn: |_table, _h| None,
            index_bits: 0,
        };
    }

    let (lookup_fn, index_bits): (fn(&ScatteredMapTable, h: u64) -> Option<u64>, u8);
    if records.len() < 256 {
        lookup_fn = lookup::<8, LinearProbe>;
        index_bits = 8;
    } else if records.len() < 65536 {
        lookup_fn = lookup::<16, LinearProbe>;
        index_bits = 16;
    } else {
        lookup_fn = lookup::<24, LinearProbe>;
        index_bits = 24;
    }

    let align = align_of::<MetadataStride>();
    let base = refs.as_mut_ptr() as usize;
    let offset = (align - (base % align)) % align;
    let num_groups = (refs.len() - offset) / std::mem::size_of::<MetadataStride>();
    let metadata = unsafe {
        core::slice::from_raw_parts_mut(
            refs.as_mut_ptr().add(offset) as *mut MetadataStride,
            num_groups,
        )
    };

    debug_assert!(num_groups > 0, "table must have at least one group");

    for (row, record) in records.iter().enumerate() {
        let hash = record.hash;
        let ctrl_byte = control_byte_from_hash(hash);

        let mut probe = LinearProbe::new(num_groups, hash);

        loop {
            let Some(g) = probe.next() else {
                panic!("no empty slot found (table full)");
            };

            let group_stride = g / BUCKET_STRIDE;
            let group_offset = g % BUCKET_STRIDE;
            let group = &mut metadata[group_stride];

            if let Some(lane) = first_empty_lane(&group.buckets[group_offset]) {
                group.buckets[group_offset].as_mut_array()[lane] = ctrl_byte;
                group.hashes[lane + BUCKET_SIZE * group_offset] = pack_hash(index_bits, hash, row);
                break;
            }
        }
    }

    ScatteredMapTable {
        metadata,
        lookup_fn,
        index_bits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{hash::ConstHasher, probe::split_hash, table::HASH_STRIDE};

    #[test]
    fn initialize_table() {
        let records = [
            MapRecord::new(
                "apple",
                1u32,
                ConstHasher::<&'static str>::const_hash("apple"),
            ),
            MapRecord::new(
                "banana",
                2u32,
                ConstHasher::<&'static str>::const_hash("banana"),
            ),
        ];

        // We know this is only ever used locally
        #[allow(static_mut_refs)]
        let refs = {
            static mut REFS: [u8; std::mem::size_of::<MetadataStride>() * 4] = [0u8; _];
            unsafe { &mut REFS }.as_mut_slice()
        };

        let table = initialize_scattered_map(&records, refs);

        let mut seen = [false; 2];
        for group in table.metadata {
            for (group_offset, bucket) in group.buckets.iter().enumerate() {
                let ctrl = bucket.as_array();
                for lane in 0..16 {
                    if ctrl[lane] == 0 {
                        continue;
                    }
                    assert_ne!(ctrl[lane] & 0x80, 0, "occupied slot must have tag bit set");
                    let (hash, offset) = split_hash(
                        table.index_bits,
                        group.hashes[group_offset * HASH_STRIDE + lane],
                    );
                    assert!(offset < 2, "bad row index: {offset}");
                    eprintln!("hash: {hash:x}, offset: {offset}");
                    assert_eq!(hash, split_hash(table.index_bits, records[offset].hash).0);
                    seen[offset] = true;
                }
            }
        }
        assert!(seen[0] && seen[1]);
    }

    // Make sure we always have enough slack space for the metadata.
    #[test]
    fn ensure_safe_record_count_for_capacity() {
        for i in 0..5000 {
            let capacity = BASE_CAPACITY + i * PER_ITEM_CAPACITY;
            let record_count = capacity / std::mem::size_of::<MetadataStride>();
            let safe_record_count = safe_record_count_for_capacity(i);
            assert!(
                safe_record_count < record_count,
                "{}: {} / {}",
                i,
                safe_record_count,
                record_count
            );
            if i % 100 == 0 {
                let actual_capacity = i as f32 / (record_count * MetadataStride::CAPACITY) as f32;
                eprintln!(
                    "{}: {} / {} ({}%)",
                    i,
                    safe_record_count,
                    record_count,
                    actual_capacity * 100.0
                );
            }
        }
    }
}
