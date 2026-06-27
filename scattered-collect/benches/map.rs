use std::{collections::HashMap, mem::MaybeUninit, ptr, sync::LazyLock};

use divan::Bencher;
use scattered_collect::{
    const_hash,
    hash_sorted_map::{
        HashBackref, MapRecord as HashSortedMapRecord, RADIX_TABLES_ZERO, RadixLookupTables,
        TAG_BLOCK_SIZE, Tag,
        internal::{
            hybrid_interpolation_search, initialize_hash_sorted_map_index_with_scratch,
            interpolation_search,
        },
    },
    map::{MapRecord, initialize_scattered_map, safe_byte_count_for_capacity},
};

const NUM_RECORDS: usize = 5000;

/// Probe keys (all present) with precomputed hash and expected value. Hashing is
/// hoisted out of the timed loops so the benches measure only the lookup, not the
/// (identical) hasher.
static PROBES: [(u64, u32); 3] = [
    (const_hash!("key0500"), 500),
    (const_hash!("key0100"), 100),
    (const_hash!("key0254"), 254),
];

/// Hashes of keys absent from the map (same shape as the present keys, but past
/// the inserted range), for the miss (`None`) path.
static MISSING_PROBES: [u64; 3] = [
    const_hash!("key5000"),
    const_hash!("key7500"),
    const_hash!("key9999"),
];

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

/// Every key, hashed, in a scrambled order (stride coprime with `NUM_RECORDS`).
/// Probing all of them per iteration spills L1 so lookups hit L2.
static SWEEP_PROBES: [(u64, u32); NUM_RECORDS] = const {
    let mut probes = [(0u64, 0u32); NUM_RECORDS];
    let mut i = 0;
    while i < NUM_RECORDS {
        let j = (i * 2503) % NUM_RECORDS;
        let Ok(s) = str::from_utf8(STRINGS[j].as_slice()) else {
            panic!("invalid string");
        };
        probes[i] = (const_hash!(s), j as u32);
        i += 1;
    }
    probes
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
    tags: [Tag; NUM_RECORDS + TAG_BLOCK_SIZE],
    radix: RadixLookupTables,
    scratch: Vec<HashBackref<&'static str, u32>>,
    counts: Vec<u32>,
}

impl HashSortedBenchState {
    fn new_unsorted() -> Self {
        Self {
            // Copy from the link-order index.
            index: std::array::from_fn(|i| {
                HashBackref::new(HASH_INDEX_UNSORTED[i].hash, HASH_INDEX_UNSORTED[i].record)
            }),
            tags: [0; NUM_RECORDS + TAG_BLOCK_SIZE],
            radix: RADIX_TABLES_ZERO,
            scratch: Vec::with_capacity(NUM_RECORDS),
            counts: Vec::new(),
        }
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
            &mut self.counts,
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
        .bench_local_values(|()| unsafe {
            initialize_scattered_map(&MAP_RECORDS, std::mem::transmute((*refs).as_mut_slice()));
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
        for (hash, n) in PROBES {
            let offset = (table.lookup_fn)(&table, divan::black_box(hash));
            let value = offset.map(|offset| &MAP_RECORDS[offset as usize].value);
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn scattered_map_lookup_sweep(bencher: Bencher) {
    let mut refs = [0_u8; safe_byte_count_for_capacity(MAP_RECORDS.len())];
    let table = initialize_scattered_map(&MAP_RECORDS, unsafe {
        std::mem::transmute(refs.as_mut_slice())
    });

    bencher.bench_local(|| {
        for (hash, n) in SWEEP_PROBES {
            let offset = (table.lookup_fn)(&table, divan::black_box(hash));
            let value = offset.map(|offset| &MAP_RECORDS[offset as usize].value);
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn scattered_map_lookup_miss(bencher: Bencher) {
    let mut refs = [0_u8; safe_byte_count_for_capacity(MAP_RECORDS.len())];
    let table = initialize_scattered_map(&MAP_RECORDS, unsafe {
        std::mem::transmute(refs.as_mut_slice())
    });

    bencher.bench_local(|| {
        for hash in MISSING_PROBES {
            let offset = (table.lookup_fn)(&table, divan::black_box(hash));
            assert_eq!(offset, None);
        }
    });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_sorted_map_build(bencher: Bencher) {
    let mut state = HashSortedBenchState::new_unsorted();
    let state = (&mut state as *mut HashSortedBenchState)
        .cast_const()
        .cast_mut();
    bencher
        .with_inputs(|| {
            // SAFETY: `bench_local_values` runs setup and timed sections sequentially
            // on the current thread.
            unsafe {
                (*state).reset_index_from_unsorted();
            }
        })
        .bench_local_values(|()| unsafe {
            (*state).init();
        });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_sorted_map_lookup(bencher: Bencher) {
    let mut state = HashSortedBenchState::new_unsorted();
    state.init();

    bencher.bench_local(|| {
        for (hash, n) in PROBES {
            let idx = hybrid_interpolation_search(
                &state.index,
                &state.tags,
                Some(&state.radix),
                divan::black_box(hash),
            );
            let value = idx.map(|idx| unsafe { &(*state.index[idx].record).value });
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_sorted_map_lookup_sweep(bencher: Bencher) {
    let mut state = HashSortedBenchState::new_unsorted();
    state.init();

    bencher.bench_local(|| {
        for (hash, n) in SWEEP_PROBES {
            let idx = hybrid_interpolation_search(
                &state.index,
                &state.tags,
                Some(&state.radix),
                divan::black_box(hash),
            );
            let value = idx.map(|idx| unsafe { &(*state.index[idx].record).value });
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_sorted_map_lookup_miss(bencher: Bencher) {
    let mut state = HashSortedBenchState::new_unsorted();
    state.init();

    bencher.bench_local(|| {
        for hash in MISSING_PROBES {
            let idx = hybrid_interpolation_search(
                &state.index,
                &state.tags,
                Some(&state.radix),
                divan::black_box(hash),
            );
            assert_eq!(idx, None);
        }
    });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_sorted_map_lookup_scalar(bencher: Bencher) {
    let mut state = HashSortedBenchState::new_unsorted();
    state.init();

    bencher.bench_local(|| {
        for (hash, n) in PROBES {
            let idx = interpolation_search(&state.index, divan::black_box(hash));
            let value = idx.map(|idx| unsafe { &(*state.index[idx].record).value });
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_map_build(bencher: Bencher) {
    let mut hash_map = HashMap::with_capacity(MAP_RECORDS.len());
    let hash_map = (&mut hash_map as *mut HashMap<&'static str, u32>)
        .cast_const()
        .cast_mut();
    bencher
        .with_inputs(|| {
            // SAFETY: `bench_local_values` runs setup and timed sections sequentially
            // on the current thread.
            unsafe {
                (*hash_map).clear();
            }
        })
        .bench_local_values(|()| unsafe {
            for record in &MAP_RECORDS {
                (*hash_map).insert(record.key, record.value);
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
    // Coarse baseline: `get` hashes internally, so this includes SipHash and is not
    // like-for-like with the hoisted scattered-map lookups.
    bencher.bench_local(|| {
        for (n, key) in [(500u32, "key0500"), (100, "key0100"), (254, "key0254")] {
            let value = hash_map.get(key);
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
#[allow(static_mut_refs)]
fn hash_map_lookup_miss(bencher: Bencher) {
    let mut hash_map = HashMap::with_capacity(MAP_RECORDS.len());
    for record in &MAP_RECORDS {
        hash_map.insert(record.key, record.value);
    }
    bencher.bench_local(|| {
        for key in ["key5000", "key7500", "key9999"] {
            assert_eq!(hash_map.get(key), None);
        }
    });
}

// 50k benches: data is generated at runtime with heap buffers.
const N_LARGE: usize = 50_000;

struct LargeData {
    map_records: Vec<MapRecord<&'static str, u32>>,
    // Owns the record storage that `hs_index_unsorted` points into; never read directly.
    #[allow(dead_code)]
    hs_records: Vec<HashSortedMapRecord<&'static str, u32>>,
    hs_index_unsorted: Vec<HashBackref<&'static str, u32>>,
    sweep: Vec<(u64, u32)>,
    hot: [(u64, u32); 3],
    miss: [u64; 3],
}

static LARGE: LazyLock<LargeData> = LazyLock::new(|| {
    let keys: Vec<&'static str> = (0..N_LARGE)
        .map(|i| Box::leak(format!("key{i:05}").into_boxed_str()) as &'static str)
        .collect();

    let map_records: Vec<MapRecord<&'static str, u32>> = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| MapRecord::new(k, i as u32, const_hash!(k)))
        .collect();

    let hs_records: Vec<HashSortedMapRecord<&'static str, u32>> = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| HashSortedMapRecord::new(k, i as u32))
        .collect();

    // Raw pointers reference the records' (stable) heap storage; moving the `Vec` into
    // `LargeData` does not move its allocation.
    let hs_index_unsorted: Vec<HashBackref<&'static str, u32>> = hs_records
        .iter()
        .map(|record| HashBackref::new(const_hash!(record.key), record as *const _))
        .collect();

    // Scrambled order (stride coprime with `N_LARGE`) so a full sweep spills cache.
    let sweep: Vec<(u64, u32)> = (0..N_LARGE)
        .map(|i| {
            let j = (i * 24_999) % N_LARGE;
            (const_hash!(keys[j]), j as u32)
        })
        .collect();

    let hot: [(u64, u32); 3] = [
        (const_hash!(keys[25_000]), 25_000),
        (const_hash!(keys[100]), 100),
        (const_hash!(keys[49_999]), 49_999),
    ];
    let miss: [u64; 3] = [
        const_hash!("key50000"),
        const_hash!("key75000"),
        const_hash!("key99999"),
    ];

    LargeData {
        map_records,
        hs_records,
        hs_index_unsorted,
        sweep,
        hot,
        miss,
    }
});

/// Heap-resident equivalent of [`HashSortedBenchState`] for the 50k benches.
struct LargeHashSortedState {
    index: Vec<HashBackref<&'static str, u32>>,
    tags: Vec<Tag>,
    radix: RadixLookupTables,
    scratch: Vec<HashBackref<&'static str, u32>>,
    counts: Vec<u32>,
}

impl LargeHashSortedState {
    fn new() -> Self {
        Self {
            index: LARGE
                .hs_index_unsorted
                .iter()
                .map(|h| HashBackref::new(h.hash, h.record))
                .collect(),
            tags: vec![0; N_LARGE + TAG_BLOCK_SIZE],
            radix: RADIX_TABLES_ZERO,
            scratch: Vec::with_capacity(N_LARGE),
            counts: Vec::new(),
        }
    }

    fn reset_index(&mut self) {
        // SAFETY: both buffers hold `N_LARGE` `HashBackref`s and do not overlap.
        unsafe {
            ptr::copy_nonoverlapping(
                LARGE.hs_index_unsorted.as_ptr(),
                self.index.as_mut_ptr(),
                N_LARGE,
            );
        }
    }

    fn init(&mut self) {
        initialize_hash_sorted_map_index_with_scratch(
            &mut self.index,
            &mut self.tags,
            &mut self.radix,
            &mut self.scratch,
            &mut self.counts,
        );
    }
}

#[divan::bench]
fn scattered_map_build_50k(bencher: Bencher) {
    let refs_len = safe_byte_count_for_capacity(N_LARGE);
    let mut refs = vec![0_u8; refs_len];
    let refs = (&mut refs as *mut Vec<u8>).cast_const().cast_mut();
    bencher
        .with_inputs(|| unsafe {
            ptr::write_bytes((*refs).as_mut_ptr(), 0, refs_len);
        })
        .bench_local_values(|()| unsafe {
            initialize_scattered_map(
                &LARGE.map_records,
                std::mem::transmute((*refs).as_mut_slice()),
            );
        });
}

#[divan::bench]
fn scattered_map_lookup_50k(bencher: Bencher) {
    let mut refs = vec![0_u8; safe_byte_count_for_capacity(N_LARGE)];
    let table = initialize_scattered_map(&LARGE.map_records, unsafe {
        std::mem::transmute(refs.as_mut_slice())
    });

    bencher.bench_local(|| {
        for &(hash, n) in &LARGE.hot {
            let offset = (table.lookup_fn)(&table, divan::black_box(hash));
            let value = offset.map(|offset| &LARGE.map_records[offset as usize].value);
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
fn scattered_map_lookup_sweep_50k(bencher: Bencher) {
    let mut refs = vec![0_u8; safe_byte_count_for_capacity(N_LARGE)];
    let table = initialize_scattered_map(&LARGE.map_records, unsafe {
        std::mem::transmute(refs.as_mut_slice())
    });

    bencher.bench_local(|| {
        for &(hash, n) in &LARGE.sweep {
            let offset = (table.lookup_fn)(&table, divan::black_box(hash));
            let value = offset.map(|offset| &LARGE.map_records[offset as usize].value);
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
fn scattered_map_lookup_miss_50k(bencher: Bencher) {
    let mut refs = vec![0_u8; safe_byte_count_for_capacity(N_LARGE)];
    let table = initialize_scattered_map(&LARGE.map_records, unsafe {
        std::mem::transmute(refs.as_mut_slice())
    });

    bencher.bench_local(|| {
        for &hash in &LARGE.miss {
            assert_eq!((table.lookup_fn)(&table, divan::black_box(hash)), None);
        }
    });
}

#[divan::bench]
fn hash_sorted_map_build_50k(bencher: Bencher) {
    let mut state = LargeHashSortedState::new();
    let state = (&mut state as *mut LargeHashSortedState)
        .cast_const()
        .cast_mut();
    bencher
        .with_inputs(|| unsafe {
            (*state).reset_index();
        })
        .bench_local_values(|()| unsafe {
            (*state).init();
        });
}

#[divan::bench]
fn hash_sorted_map_lookup_50k(bencher: Bencher) {
    let mut state = LargeHashSortedState::new();
    state.init();

    bencher.bench_local(|| {
        for &(hash, n) in &LARGE.hot {
            let idx = hybrid_interpolation_search(
                &state.index,
                &state.tags,
                Some(&state.radix),
                divan::black_box(hash),
            );
            let value = idx.map(|idx| unsafe { &(*state.index[idx].record).value });
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
fn hash_sorted_map_lookup_sweep_50k(bencher: Bencher) {
    let mut state = LargeHashSortedState::new();
    state.init();

    bencher.bench_local(|| {
        for &(hash, n) in &LARGE.sweep {
            let idx = hybrid_interpolation_search(
                &state.index,
                &state.tags,
                Some(&state.radix),
                divan::black_box(hash),
            );
            let value = idx.map(|idx| unsafe { &(*state.index[idx].record).value });
            assert_eq!(value, Some(&n));
        }
    });
}

#[divan::bench]
fn hash_sorted_map_lookup_miss_50k(bencher: Bencher) {
    let mut state = LargeHashSortedState::new();
    state.init();

    bencher.bench_local(|| {
        for &hash in &LARGE.miss {
            let idx = hybrid_interpolation_search(
                &state.index,
                &state.tags,
                Some(&state.radix),
                divan::black_box(hash),
            );
            assert_eq!(idx, None);
        }
    });
}

fn main() {
    divan::main();
}
