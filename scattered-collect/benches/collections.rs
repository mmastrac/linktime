//! Benchmarks for the public, link-time-built collections.
//!
//! These benchmarks exercise the runtime-facing operations of the collections
//! (lookups, iteration, hashing and sorted-slice initialization), and compare
//! the map/set lookups against the equivalent `std` collections.

use std::collections::{HashMap, HashSet};

use divan::{Bencher, black_box};
use scattered_collect::{
    ScatteredMap, ScatteredSet, gather,
    hash::ConstHash,
    scatter,
    sorted_slice::{ScatteredSortedSlice, initialize_scattered_sorted_slice},
};

/// A link-time registry of plugin names, in the shape most users would build.
#[gather]
static PLUGINS: ScatteredMap<&'static str, u32>;

/// The same keys, gathered into a set.
#[gather]
static PLUGIN_NAMES: ScatteredSet<&'static str>;

/// A link-time sorted slice, sorted in a priority-0 constructor.
#[gather]
static PLUGIN_IDS: ScatteredSortedSlice<u32>;

/// Generate one scatter site per identifier in each collection.
macro_rules! scatter_plugins {
    ($($n:literal)*) => {
        $(
            #[scatter(PLUGINS)]
            const _: (&'static str, u32) = (concat!("plugin_", $n), $n);

            #[scatter(PLUGIN_NAMES)]
            const _: &'static str = concat!("plugin_", $n);

            #[scatter(PLUGIN_IDS)]
            const _: u32 = $n;
        )*
    };
}

scatter_plugins!(
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15
    16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
    32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47
    48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
    64 65 66 67 68 69 70 71 72 73 74 75 76 77 78 79
    80 81 82 83 84 85 86 87 88 89 90 91 92 93 94 95
    96 97 98 99 100 101 102 103 104 105 106 107 108 109 110 111
    112 113 114 115 116 117 118 119 120 121 122 123 124 125 126 127
);

/// The keys that are present in the collections above.
const HIT_KEYS: [&str; 8] = [
    "plugin_0",
    "plugin_7",
    "plugin_23",
    "plugin_42",
    "plugin_64",
    "plugin_99",
    "plugin_120",
    "plugin_127",
];

/// Keys that are absent from the collections above.
const MISS_KEYS: [&str; 8] = [
    "plugin_128",
    "plugin_1000",
    "unknown",
    "plugin",
    "plugin_-1",
    "plugin_0x0",
    "plugin_00",
    "",
];

fn std_map() -> HashMap<&'static str, u32> {
    PLUGINS.entries().map(|(k, v)| (*k, *v)).collect()
}

fn std_set() -> HashSet<&'static str> {
    PLUGINS.keys().copied().collect()
}

mod map {
    use super::*;

    /// Lookup of keys that are present in the link-time map.
    #[divan::bench]
    fn scattered_get_hit(bencher: Bencher) {
        bencher.bench(|| {
            for key in black_box(&HIT_KEYS) {
                black_box(PLUGINS.get(key));
            }
        });
    }

    /// Lookup of keys that are absent from the link-time map.
    #[divan::bench]
    fn scattered_get_miss(bencher: Bencher) {
        bencher.bench(|| {
            for key in black_box(&MISS_KEYS) {
                black_box(PLUGINS.get(key));
            }
        });
    }

    /// Baseline: the same lookups against a `std` `HashMap`.
    #[divan::bench]
    fn std_hash_map_get_hit(bencher: Bencher) {
        let map = std_map();
        bencher.bench_local(|| {
            for key in black_box(&HIT_KEYS) {
                black_box(map.get(key));
            }
        });
    }

    /// Baseline: the same misses against a `std` `HashMap`.
    #[divan::bench]
    fn std_hash_map_get_miss(bencher: Bencher) {
        let map = std_map();
        bencher.bench_local(|| {
            for key in black_box(&MISS_KEYS) {
                black_box(map.get(key));
            }
        });
    }

    /// Lookup every key of the map, once.
    #[divan::bench]
    fn scattered_get_all(bencher: Bencher) {
        let keys: Vec<&'static str> = PLUGINS.keys().copied().collect();
        bencher.bench_local(|| {
            for key in black_box(&keys) {
                black_box(PLUGINS.get(key));
            }
        });
    }

    /// Full iteration over the gathered records.
    #[divan::bench]
    fn scattered_iterate(bencher: Bencher) {
        bencher.bench(|| {
            let mut sum = 0u64;
            for (key, value) in &PLUGINS {
                sum += key.len() as u64 + *value as u64;
            }
            black_box(sum)
        });
    }
}

mod set {
    use super::*;

    /// Membership checks that hit.
    #[divan::bench]
    fn scattered_contains_hit(bencher: Bencher) {
        bencher.bench(|| {
            for key in black_box(&HIT_KEYS) {
                black_box(PLUGIN_NAMES.contains(key));
            }
        });
    }

    /// Membership checks that miss.
    #[divan::bench]
    fn scattered_contains_miss(bencher: Bencher) {
        bencher.bench(|| {
            for key in black_box(&MISS_KEYS) {
                black_box(PLUGIN_NAMES.contains(key));
            }
        });
    }

    /// Baseline: the same membership checks against a `std` `HashSet`.
    #[divan::bench]
    fn std_hash_set_contains_hit(bencher: Bencher) {
        let set = std_set();
        bencher.bench_local(|| {
            for key in black_box(&HIT_KEYS) {
                black_box(set.contains(key));
            }
        });
    }
}

mod sorted_slice {
    use super::*;

    /// Binary search over the link-time sorted slice.
    #[divan::bench]
    fn binary_search(bencher: Bencher) {
        bencher.bench(|| {
            for id in [0u32, 7, 23, 42, 64, 99, 120, 127, 200] {
                black_box(PLUGIN_IDS.binary_search(&black_box(id)).ok());
            }
        });
    }

    /// Iteration over the link-time sorted slice.
    #[divan::bench]
    fn iterate(bencher: Bencher) {
        bencher.bench(|| {
            let mut sum = 0u64;
            for id in &*PLUGIN_IDS {
                sum += *id as u64;
            }
            black_box(sum)
        });
    }

    /// The sorting work performed by the priority-0 constructor, for a slice
    /// gathered in a pseudo-random link order.
    #[divan::bench(args = [128, 4096])]
    fn initialize(bencher: Bencher, len: usize) {
        // A deterministic, badly-ordered input.
        let data: Vec<u32> = (0..len as u32)
            .map(|i| i.wrapping_mul(2654435761) >> 3)
            .collect();
        bencher
            .with_inputs(|| data.clone())
            .bench_local_refs(|slice| initialize_scattered_sorted_slice(slice.as_mut_slice()));
    }
}

mod hash {
    use super::*;

    /// The runtime side of `const_hash` for short keys.
    #[divan::bench]
    fn hash_short_str(bencher: Bencher) {
        bencher.bench(|| {
            for key in black_box(&HIT_KEYS) {
                black_box(ConstHash::hash(key));
            }
        });
    }

    /// The runtime side of `const_hash` for a long key.
    #[divan::bench]
    fn hash_long_str(bencher: Bencher) {
        let key = "a_rather_long_plugin_identifier_that_does_not_fit_in_a_single_word".repeat(4);
        bencher.bench_local(|| black_box(ConstHash::hash(black_box(key.as_str()))));
    }

    /// The runtime side of `const_hash` for numeric keys.
    #[divan::bench]
    fn hash_u64(bencher: Bencher) {
        bencher.bench(|| {
            for value in [0u64, 1, 42, u64::MAX] {
                black_box(ConstHash::hash(&black_box(value)));
            }
        });
    }
}

fn main() {
    divan::main();
}
