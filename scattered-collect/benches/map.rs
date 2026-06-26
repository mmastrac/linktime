use std::{collections::HashMap, mem::MaybeUninit, ptr};

use divan::Bencher;
use scattered_collect::{
    const_hash,
    hash_sorted_map::{
        HashBackref, MapRecord as HashSortedMapRecord, RadixLookupTables, TAG_BLOCK_SIZE,
        TOP_BYTE_INDEX_ZERO, hybrid_interpolation_search,
        initialize_hash_sorted_map_index_with_scratch, interpolation_search,
    },
    map::{MapRecord, initialize_scattered_map, safe_byte_count_for_capacity},
};

const NUM_RECORDS: usize = 5000;

const fn make_static_string(i: usize) -> [u8; 7] {
    let mut s = [0u8; 7];
    s[0] = b'k';
    s[1] = b'e';
    s[2] = b'y';
    s[3] = b'0' + (i / 1000) as u8;
    let i = i % 1000;
    s[4] = b'0' + (i / 100) as u8;
    let i = i % 100;
    s[5] = b'0' + (i / 10) as u8;
    let i = i % 10;
    s[6] = b'0' + i as u8;
    s
}

static STRINGS: [[u8; 7]; NUM_RECORDS] = const {
    let mut strings = [[0u8; 7]; NUM_RECORDS];
    let mut i = 0;
    while i < NUM_RECORDS {
        strings[i] = make_static_string(i);
        i += 1;
    }
    strings
};

static MAP_RECORDS: [MapRecord<&'static str, u32>; NUM_RECORDS] = const {
    let mut records: [MaybeUninit<MapRecord<&'static str, u32>>; NUM_RECORDS] = unsafe {
        std::mem::transmute(MaybeUninit::<
            [MaybeUninit<MapRecord<&'static str, u32>>; NUM_RECORDS],
        >::uninit())
    };
    let mut i = 0;
    while i < NUM_RECORDS {
        let Ok(s) = str::from_utf8(STRINGS[i].as_slice()) else {
            panic!("invalid string");
        };
        records[i] = MaybeUninit::new(MapRecord::new(s, i as u32, const_hash!(s)));
        i += 1;
    }
    unsafe { std::mem::transmute(records) }
};

static HASH_SORTED_RECORDS: [HashSortedMapRecord<&'static str, u32>; NUM_RECORDS] = const {
    let mut records: [MaybeUninit<HashSortedMapRecord<&'static str, u32>>; NUM_RECORDS] = unsafe {
        std::mem::transmute(MaybeUninit::<
            [MaybeUninit<HashSortedMapRecord<&'static str, u32>>; NUM_RECORDS],
        >::uninit())
    };
    let mut i = 0;
    while i < NUM_RECORDS {
        let Ok(s) = str::from_utf8(STRINGS[i].as_slice()) else {
            panic!("invalid string");
        };
        records[i] = MaybeUninit::new(HashSortedMapRecord::new(s, i as u32));
        i += 1;
    }
    unsafe { std::mem::transmute(records) }
};

static HASH_INDEX_UNSORTED: [HashBackref<&'static str, u32>; NUM_RECORDS] = const {
    let mut index: [MaybeUninit<HashBackref<&'static str, u32>>; NUM_RECORDS] = unsafe {
        std::mem::transmute(MaybeUninit::<
            [MaybeUninit<HashBackref<&'static str, u32>>; NUM_RECORDS],
        >::uninit())
    };
    let mut i = 0;
    while i < NUM_RECORDS {
        let Ok(s) = str::from_utf8(STRINGS[i].as_slice()) else {
            panic!("invalid string");
        };
        index[i] = MaybeUninit::new(HashBackref::new(
            const_hash!(s),
            &HASH_SORTED_RECORDS[i] as *const HashSortedMapRecord<&'static str, u32>,
        ));
        i += 1;
    }
    unsafe { std::mem::transmute(index) }
};

/// Mutable link-section-like buffers for hash-sorted map benches.
struct HashSortedBenchState {
    index: [HashBackref<&'static str, u32>; NUM_RECORDS],
    tags: [u16; NUM_RECORDS + TAG_BLOCK_SIZE],
    radix: RadixLookupTables,
    scratch: Vec<HashBackref<&'static str, u32>>,
}

impl HashSortedBenchState {
    fn new_unsorted() -> Self {
        let mut state = Self {
            index: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            tags: [0_u16; NUM_RECORDS + TAG_BLOCK_SIZE],
            radix: TOP_BYTE_INDEX_ZERO,
            scratch: Vec::with_capacity(NUM_RECORDS),
        };
        state.reset_index_from_unsorted();
        state
    }

    /// Restore the index section to link-order data so init can run again.
    ///
    /// Production runs this path once on the gathered section with no copy.
    /// The bench calls this outside the timed section via [`divan::Bencher::with_inputs`].
    fn reset_index_from_unsorted(&mut self) {
        unsafe {
            ptr::copy_nonoverlapping(
                HASH_INDEX_UNSORTED.as_ptr(),
                self.index.as_mut_ptr(),
                NUM_RECORDS,
            );
        }
    }

    fn init(&mut self) {
        initialize_hash_sorted_map_index_with_scratch(
            &mut self.index,
            &mut self.tags,
            &mut self.radix,
            &mut self.scratch,
        );
    }
}

#[divan::bench]
#[allow(static_mut_refs)]
fn scattered_map_build(bencher: Bencher) {
    const REFS_LEN: usize = safe_byte_count_for_capacity(NUM_RECORDS);
    let mut refs = [0_u8; REFS_LEN];
    let refs = (&mut refs as *mut [u8; REFS_LEN]).cast_const().cast_mut();
    bencher
        .with_inputs(|| {
            // SAFETY: `bench_local_values` runs setup and timed sections sequentially
            // on the current thread.
            unsafe {
                ptr::write_bytes((*refs).as_mut_ptr(), 0, REFS_LEN);
            }
        })
        .bench_local_values(|()| {
            unsafe {
                initialize_scattered_map(&MAP_RECORDS, std::mem::transmute((*refs).as_mut_slice()));
            }
        });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn scattered_map_lookup(bencher: Bencher) {
    let mut refs = [0_u8; safe_byte_count_for_capacity(MAP_RECORDS.len())];
    let table = initialize_scattered_map(&MAP_RECORDS, unsafe {
        std::mem::transmute(refs.as_mut_slice())
    });

    bencher.bench_local(|| {
        for (n, key) in [(500, "key0500"), (100, "key0100"), (254, "key0254")] {
            let hash = const_hash!(key);
            let offset = (table.lookup_fn)(&table, hash);
            let value = offset.map(|offset| &MAP_RECORDS[offset as usize].value);
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_sorted_map_build(bencher: Bencher) {
    let mut state = HashSortedBenchState::new_unsorted();
    let state = (&mut state as *mut HashSortedBenchState).cast_const().cast_mut();
    bencher
        .with_inputs(|| {
            // SAFETY: `bench_local_values` runs setup and timed sections sequentially
            // on the current thread.
            unsafe {
                (*state).reset_index_from_unsorted();
            }
        })
        .bench_local_values(|()| {
            unsafe {
                (*state).init();
            }
        });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_sorted_map_lookup(bencher: Bencher) {
    let mut state = HashSortedBenchState::new_unsorted();
    state.init();

    bencher.bench_local(|| {
        for (n, key) in [(500, "key0500"), (100, "key0100"), (254, "key0254")] {
            let hash = const_hash!(key);
            let idx = hybrid_interpolation_search(
                &state.index,
                &state.tags,
                Some(&state.radix),
                hash,
            );
            let value = idx.map(|idx| unsafe { &(*state.index[idx].record).value });
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_sorted_map_lookup_scalar(bencher: Bencher) {
    let mut state = HashSortedBenchState::new_unsorted();
    state.init();

    bencher.bench_local(|| {
        for (n, key) in [(500, "key0500"), (100, "key0100"), (254, "key0254")] {
            let hash = const_hash!(key);
            let idx = interpolation_search(&state.index, hash);
            let value = idx.map(|idx| unsafe { &(*state.index[idx].record).value });
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_map_build(bencher: Bencher) {
    let mut hash_map = HashMap::with_capacity(MAP_RECORDS.len());
    let hash_map = (&mut hash_map as *mut HashMap<&'static str, u32>).cast_const().cast_mut();
    bencher
        .with_inputs(|| {
            // SAFETY: `bench_local_values` runs setup and timed sections sequentially
            // on the current thread.
            unsafe {
                (*hash_map).clear();
            }
        })
        .bench_local_values(|()| {
            unsafe {
                for record in &MAP_RECORDS {
                    (*hash_map).insert(record.key, record.value);
                }
            }
        });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_map_lookup(bencher: Bencher) {
    let mut hash_map = HashMap::with_capacity(MAP_RECORDS.len());
    for record in &MAP_RECORDS {
        hash_map.insert(record.key, record.value);
    }
    bencher.bench_local(|| {
        for (n, key) in [(500, "key0500"), (100, "key0100"), (254, "key0254")] {
            let value = hash_map.get(key);
            assert_eq!(value, Some(&n));
        }
    });
}

fn main() {
    // Run registered benchmarks.
    divan::main();
}
