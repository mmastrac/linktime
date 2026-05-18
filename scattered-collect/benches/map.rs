use std::{collections::HashMap, mem::MaybeUninit};

use divan::Bencher;
use scattered_collect::{
    const_hash,
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

#[divan::bench]
#[allow(static_mut_refs)]
fn scattered_map_build(bencher: Bencher) {
    bencher.bench_local(|| {
        let mut refs = [0_u8; safe_byte_count_for_capacity(MAP_RECORDS.len())];
        initialize_scattered_map(&MAP_RECORDS, unsafe {
            std::mem::transmute(refs.as_mut_slice())
        });
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
fn hash_map_build(bencher: Bencher) {
    let mut hash_map = HashMap::with_capacity(MAP_RECORDS.len());
    bencher.bench_local(|| {
        for record in &MAP_RECORDS {
            hash_map.insert(record.key, record.value);
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
