#![allow(clippy::modulo_one, unreachable_pub)]
use crate::map::MapRecord;
use crate::map::probe::{
    BUCKET_SIZE, LinearProbe, ProbeStrategy, control_byte_from_hash, first_empty_lane, match_mask,
    split_hash,
};
use crate::map::table::{BUCKET_STRIDE, MetadataStride, ScatteredMapTable};

pub const SAFE_CAPACITY: f32 = 0.70;
pub const BASE_CAPACITY: usize = 512;
pub const PER_ITEM_CAPACITY: usize = 14;

const _: () = assert!(
    BASE_CAPACITY >= 3 * size_of::<MetadataStride>() + align_of::<MetadataStride>(),
    "BASE_CAPACITY too small for METADATA_STRIDE"
);

#[allow(unused)]
pub const fn safe_record_count_for_capacity(capacity: usize) -> usize {
    (capacity * 100 / ((SAFE_CAPACITY * 100.0) as usize)) / MetadataStride::CAPACITY + 2
}

/// The number of bytes required to store a scattered map table with the given
/// capacity and a safe fill ratio.
#[allow(unused)]
pub const fn safe_byte_count_for_capacity(capacity: usize) -> usize {
    safe_record_count_for_capacity(capacity) * std::mem::size_of::<MetadataStride>()
}

pub const fn pack_hash(index_bits: u8, hash: u64, index: usize) -> u64 {
    let hash_mask: u64 = (-1_i64 as u64) << (index_bits as usize);
    let index_mask = !hash_mask;
    debug_assert!(index as u64 <= index_mask);
    hash & hash_mask | (index as u64) & index_mask
}

/// Initialize a scattered map table from a slice of records.
#[allow(unsafe_code)]
#[doc(hidden)]
pub fn initialize_scattered_map<K, V>(
    records: &[MapRecord<K, V>],
    refs: &'static mut [u8],
) -> ScatteredMapTable {
    let n = records.len();
    if n == 0 {
        return ScatteredMapTable {
            metadata: &[],
            index_bits: 0,
        };
    }

    let index_bits = if records.len() < 256 {
        8
    } else if records.len() < 65536 {
        16
    } else {
        24
    };

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

        let mut probe = LinearProbe::new(num_groups * BUCKET_STRIDE, hash);

        loop {
            let Some(g) = probe.next() else {
                panic!("no empty slot found: table full after inserting {row} record(s)");
            };

            let group_stride = g / BUCKET_STRIDE;
            let group_offset = g % BUCKET_STRIDE;
            let group = &mut metadata[group_stride];

            // First, check for duplicate hash
            let mut bits = match_mask(&group.buckets[group_offset], ctrl_byte);
            while bits != 0 {
                let lane = bits.trailing_zeros() as usize;
                let h2 = group.hashes[group_offset * BUCKET_SIZE + lane];
                let (stored, index) = split_hash(index_bits, h2);
                let hash_mask = (-1_i64 as u64) << (index_bits as usize);
                if stored == hash & hash_mask {
                    panic!("duplicate hash found: {stored:x} at index {index}");
                }
                bits &= bits - 1;
            }

            if let Some(lane) = first_empty_lane(&group.buckets[group_offset]) {
                group.buckets[group_offset].as_mut_array()[lane] = ctrl_byte;
                group.hashes[lane + BUCKET_SIZE * group_offset] = pack_hash(index_bits, hash, row);
                break;
            }
        }
    }

    ScatteredMapTable {
        metadata,
        index_bits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{const_hash, map::probe::split_hash};

    #[test]
    fn initialize_table() {
        let records = [
            MapRecord::new("apple", 1u32, const_hash!("apple")),
            MapRecord::new("banana", 2u32, const_hash!("banana")),
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
                for lane in 0..BUCKET_SIZE {
                    if ctrl[lane] == 0 {
                        continue;
                    }
                    assert_ne!(ctrl[lane] & 0x80, 0, "occupied slot must have tag bit set");
                    let (hash, offset) = split_hash(
                        table.index_bits,
                        group.hashes[group_offset * BUCKET_SIZE + lane],
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

    #[allow(unsafe_code)]
    fn round_trip(n: usize, strides: usize) {
        let mut buffer: Vec<MetadataStride> = (0..strides).map(|_| MetadataStride::ZERO).collect();
        let refs: &'static mut [u8] = unsafe {
            core::slice::from_raw_parts_mut(
                buffer.as_mut_ptr().cast::<u8>(),
                strides * std::mem::size_of::<MetadataStride>(),
            )
        };

        let keys: Vec<String> = (0..n).map(|i| format!("key{i:08}")).collect();
        let records: Vec<MapRecord<&str, usize>> = keys
            .iter()
            .enumerate()
            .map(|(i, k)| MapRecord::new(k.as_str(), i, crate::hash::ConstHash::hash(&k.as_str())))
            .collect();

        let table = initialize_scattered_map(&records, refs);

        for (i, key) in keys.iter().enumerate() {
            let found = table.lookup(crate::hash::ConstHash::hash(&key.as_str()));
            assert!(found.is_found(), "n={n} strides={strides}: missed {key}");
            assert_eq!(found.unwrap() as usize, i, "n={n} strides={strides}: {key}");
        }

        for i in 0..if cfg!(miri) { 50 } else { 1000 } {
            let key = format!("absent{i:08}");
            let found = table.lookup(crate::hash::ConstHash::hash(&key.as_str()));
            if found.is_found() {
                assert!(
                    (found.unwrap() as usize) < n,
                    "n={n} strides={strides}: out-of-range index for {key}"
                );
            }
        }
    }

    const ROUND_TRIP_SIZES: &[usize] = if cfg!(miri) {
        &[1, 2, 16, 17, 255, 256, 257]
    } else if cfg!(bsan) {
        &[1, 2, 15, 16, 17, 255, 256, 257, 1000]
    } else {
        &[
            1, 2, 15, 16, 17, 255, 256, 257, 1000, 5000, 65535, 65536, 70000,
        ]
    };

    #[test]
    fn lookup_round_trip_safe_capacity() {
        for &n in ROUND_TRIP_SIZES {
            round_trip(n, safe_record_count_for_capacity(n));
        }
    }

    #[test]
    fn lookup_round_trip_full_table() {
        for &n in ROUND_TRIP_SIZES {
            round_trip(n, n.div_ceil(MetadataStride::CAPACITY));
        }
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
