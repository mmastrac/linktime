//! A hash-sorted map initialized with link-time data.
#![doc = concat!("```rust\n", include_str!("../examples/hash_sorted_map.rs"), "\n```\n")]

use link_section::{TypedMutableSection, TypedSection};
use wide::{CmpEq, u16x16};

use crate::hash::ConstHash;

/// The number of `u16` tags compared per SIMD step.
pub const TAG_BLOCK_SIZE: usize = 16;

/// Below this radix-bucket width, skip interpolation and linearly scan tags.
pub const RADIX_LINEAR_THRESHOLD: usize = 64;

/// Zero `u16` tag slots reserved at gather time so the final 16-wide SIMD window
/// can load past the last record without branching.
#[doc(hidden)]
pub const TAG_GATHER_PADDING: [u16; TAG_BLOCK_SIZE] = [0; TAG_BLOCK_SIZE];

/// Per-scatter tag placeholder written before init fills the real tag.
#[doc(hidden)]
pub const TAG_SCATTER_ZERO: u16 = 0;

/// Number of top-byte partition slots (`0..=256`, with `starts[256] == len`).
pub const TOP_BYTE_SLOTS: usize = 257;

/// Precomputed radix metadata: exact bucket starts and interpolation reciprocals.
#[repr(C)]
pub struct RadixLookupTables {
    /// Exact partition start for each top hash byte; `starts[256] == len`.
    pub starts: [u16; TOP_BYTE_SLOTS],
    /// Per-bucket `floor(range * 2^64 / span)` for multiply-based interpolation.
    pub interp_mul: [u64; 256],
}

impl RadixLookupTables {
    /// Zero-filled tables written at gather and populated during initialization.
    pub const ZERO: Self = Self {
        starts: [0; TOP_BYTE_SLOTS],
        interp_mul: [0; 256],
    };
}

/// Legacy alias for the precomputed top-byte partition starts.
pub type TopByteIndex = [u16; TOP_BYTE_SLOTS];

/// Zero-filled radix tables written at gather.
#[doc(hidden)]
pub const TOP_BYTE_INDEX_ZERO: RadixLookupTables = RadixLookupTables::ZERO;

/// One gathered map entry.
#[repr(C)]
pub struct MapRecord<K, V> {
    /// The key of the record.
    pub key: K,
    /// The value of the record.
    pub value: V,
}

impl<K, V> MapRecord<K, V> {
    /// Create a new map record with a key and value.
    pub const fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

impl<K, V> ::core::fmt::Debug for MapRecord<K, V>
where
    K: ::core::fmt::Debug,
    V: ::core::fmt::Debug,
{
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("MapRecord")
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}

impl<K: 'static, V: 'static> crate::ScatteredElementType for ScatteredHashSortedMap<K, V> {
    type T = MapRecord<K, V>;
}

impl<K: 'static, V: 'static> crate::ScatteredElementTuple for ScatteredHashSortedMap<K, V> {
    type A = K;
    type B = V;
}

/// One hash-index entry pointing back to a [`MapRecord`].
#[repr(C)]
pub struct HashBackref<K, V> {
    /// The hash of the record's key.
    pub hash: u64,
    /// A pointer to the record in the records section.
    pub record: *const MapRecord<K, V>,
}

impl<K, V> HashBackref<K, V> {
    /// Create a new hash backref with a hash and record pointer.
    pub const fn new(hash: u64, record: *const MapRecord<K, V>) -> Self {
        Self { hash, record }
    }
}

// The record pointers refer to `static` link-section storage that is fixed after link time.
unsafe impl<K: Sync, V: Sync> Sync for HashBackref<K, V> {}
unsafe impl<K: Send, V: Send> Send for HashBackref<K, V> {}

/// A map whose hash index is sorted at link time for hybrid interpolation + SIMD lookup.
///
/// Scatter sites contribute a [`MapRecord`], a matching [`HashBackref`], and a zero
/// `u16` tag placeholder. A priority-0 constructor sorts the hash index and writes
/// the lower 16 bits of each sorted hash into the tag section.
///
/// For swiss-table lookup, use [`crate::ScatteredMap`]. For a sorted slice without
/// key lookup, use [`crate::ScatteredSortedSlice`].
/// ```rust
#[doc = include_str!("../examples/hash_sorted_map.rs")]
/// ```
pub struct ScatteredHashSortedMap<K: 'static, V: 'static> {
    records: &'static TypedSection<MapRecord<K, V>>,
    index: &'static TypedMutableSection<HashBackref<K, V>>,
    tags: &'static TypedMutableSection<u16>,
    radix: &'static TypedMutableSection<RadixLookupTables>,
}

impl<K: 'static, V: 'static> ScatteredHashSortedMap<K, V> {
    #[doc(hidden)]
    #[allow(unsafe_code)]
    pub const unsafe fn new(
        records: &'static TypedSection<MapRecord<K, V>>,
        index: &'static TypedMutableSection<HashBackref<K, V>>,
        tags: &'static TypedMutableSection<u16>,
        radix: &'static TypedMutableSection<RadixLookupTables>,
    ) -> Self {
        Self {
            records,
            index,
            tags,
            radix,
        }
    }

    /// Lookup a value by key using coarse interpolation jumps and SIMD tag filtering.
    #[inline]
    pub fn find(&self, key: &K) -> Option<&V>
    where
        K: ConstHash + PartialEq,
    {
        let hash = ConstHash::hash(key);
        let idx = hybrid_interpolation_search(
            self.index.as_slice(),
            self.tags.as_slice(),
            Some(radix_tables(self.radix.as_slice())),
            hash,
        )?;
        let record = unsafe { &*self.index.as_slice()[idx].record };
        if record.key == *key {
            Some(&record.value)
        } else {
            None
        }
    }

    /// True when a key is present in the map.
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool
    where
        K: ConstHash + PartialEq,
    {
        self.find(key).is_some()
    }

    /// The gathered records in link order.
    #[inline]
    pub fn records(&self) -> &[MapRecord<K, V>] {
        self.records.as_slice()
    }

    /// The hash index sorted by hash.
    #[inline]
    pub fn hash_index(&self) -> &[HashBackref<K, V>] {
        self.index.as_slice()
    }

    /// The SIMD tag slice. The first [`Self::len`] entries hold the lower 16 bits of
    /// each sorted hash; trailing entries are zero padding for safe SIMD loads.
    #[inline]
    pub fn tags(&self) -> &[u16] {
        self.tags.as_slice()
    }

    /// Precomputed radix partition starts and interpolation reciprocals.
    #[inline]
    pub fn radix_tables(&self) -> &RadixLookupTables {
        radix_tables(self.radix.as_slice())
    }

    /// Exact top-byte partition starts (`starts[256] == len`).
    #[inline]
    pub fn top_byte_index(&self) -> &TopByteIndex {
        &self.radix_tables().starts
    }

    /// Borrow keys from the record section in link order.
    pub fn keys_slice(&self) -> impl Iterator<Item = &K> {
        self.records().iter().map(|record| &record.key)
    }

    /// Borrow values from the record section in link order.
    pub fn values_slice(&self) -> impl Iterator<Item = &V> {
        self.records().iter().map(|record| &record.value)
    }

    /// Iterate over entries in link order.
    pub fn entries(&self) -> impl Iterator<Item = (&K, &V)> {
        self.records()
            .iter()
            .map(|record| (&record.key, &record.value))
    }

    /// Iterate over entries in hash-sorted order.
    pub fn sorted_entries(&self) -> impl Iterator<Item = (&K, &V)> {
        self.hash_index().iter().map(|entry| {
            let record = unsafe { &*entry.record };
            (&record.key, &record.value)
        })
    }

    /// The number of records in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// True if the map has no records.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// The offset of the record in the map, if it is from this map.
    #[inline]
    pub fn offset_of(this: &Self, record: &MapRecord<K, V>) -> Option<usize> {
        this.records.offset_of(record)
    }
}

impl<K: 'static, V: 'static> IntoIterator for &'static ScatteredHashSortedMap<K, V> {
    type Item = (&'static K, &'static V);
    type IntoIter = ::core::iter::Map<
        ::core::slice::Iter<'static, MapRecord<K, V>>,
        fn(&MapRecord<K, V>) -> (&K, &V),
    >;
    fn into_iter(self) -> Self::IntoIter {
        self.records()
            .iter()
            .map(|record| (&record.key, &record.value))
    }
}

/// The lower 16 bits of a hash used for SIMD tag filtering.
#[inline]
pub const fn hash_tag(hash: u64) -> u16 {
    hash as u16
}

/// Per-phase step counts for search analysis.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SearchStepCounts {
    /// Coarse interpolation jumps before the SIMD phase.
    pub coarse_steps: u32,
    /// Top-byte window expansion steps after the first jump.
    pub top_byte_expand_steps: u32,
    /// 16-wide SIMD blocks examined.
    pub simd_blocks: u32,
    /// SIMD tag matches that required a full-hash check.
    pub simd_candidates: u32,
    /// Scalar tag comparisons in a small radix bucket.
    pub linear_tag_checks: u32,
}

/// Sort the hash index and write sorted SIMD tags and the top-byte jump table.
pub fn initialize_hash_sorted_map_index<K, V>(
    index: &mut [HashBackref<K, V>],
    tags: &mut [u16],
    radix: &mut RadixLookupTables,
) {
    let len = index.len();
    if len == 0 {
        *radix = RadixLookupTables::ZERO;
        return;
    }
    if len > tags.len() {
        panic!(
            "tag section must have at least as many entries as the hash index (need {len})"
        );
    }

    let mut scratch = ::std::vec::Vec::with_capacity(len);
    sort_index_by_top_byte_bucket(index, radix, &mut scratch);
    finish_sorted_index(index, tags, radix);
}

/// Like [`initialize_hash_sorted_map_index`], but reuses `scratch` for the scatter buffer.
#[doc(hidden)]
pub fn initialize_hash_sorted_map_index_with_scratch<K, V>(
    index: &mut [HashBackref<K, V>],
    tags: &mut [u16],
    radix: &mut RadixLookupTables,
    scratch: &mut ::std::vec::Vec<HashBackref<K, V>>,
) {
    let len = index.len();
    if len == 0 {
        *radix = RadixLookupTables::ZERO;
        return;
    }
    if len > tags.len() {
        panic!(
            "tag section must have at least as many entries as the hash index (need {len})"
        );
    }

    sort_index_by_top_byte_bucket(index, radix, scratch);
    finish_sorted_index(index, tags, radix);
}

/// Count by top hash byte, scatter into bucket ranges, then sort each bucket slice.
#[allow(unsafe_code)]
fn sort_index_by_top_byte_bucket<K, V>(
    index: &mut [HashBackref<K, V>],
    radix: &mut RadixLookupTables,
    scratch: &mut ::std::vec::Vec<HashBackref<K, V>>,
) {
    let len = index.len();
    if len > u16::MAX as usize {
        panic!("ScatteredHashSortedMap supports at most {} records", u16::MAX);
    }

    let mut count = [0u32; 256];
    for entry in index.iter() {
        count[(entry.hash >> 56) as usize] += 1;
    }

    let mut pos = 0_u16;
    for (bucket, start) in radix.starts.iter_mut().enumerate().take(256) {
        *start = pos;
        pos += count[bucket] as u16;
    }
    radix.starts[256] = len as u16;

    scratch.clear();
    scratch.reserve(len);
    unsafe {
        ::core::ptr::copy_nonoverlapping(index.as_ptr(), scratch.as_mut_ptr(), len);
        scratch.set_len(len);
    }

    let mut cursors = radix.starts;
    for entry in scratch.iter() {
        let bucket = (entry.hash >> 56) as usize;
        let dest = cursors[bucket] as usize;
        unsafe {
            ::core::ptr::write(
                index.get_unchecked_mut(dest),
                ::core::ptr::read(entry as *const HashBackref<K, V>),
            );
        }
        cursors[bucket] += 1;
    }

    for bucket in 0..256 {
        let lo = radix.starts[bucket] as usize;
        let hi = radix.starts[bucket + 1] as usize;
        if hi > lo + 1 {
            index[lo..hi].sort_unstable_by_key(|entry| entry.hash);
        }
    }
}

/// Write SIMD tags and interpolation reciprocals for a sorted `index`.
///
/// `radix.starts` must already reflect top-byte partition boundaries.
fn finish_sorted_index<K, V>(
    index: &[HashBackref<K, V>],
    tags: &mut [u16],
    radix: &mut RadixLookupTables,
) {
    let len = index.len();
    if len > u16::MAX as usize {
        panic!("ScatteredHashSortedMap supports at most {} records", u16::MAX);
    }

    for (tag, entry) in tags.iter_mut().zip(index.iter()).take(len) {
        *tag = hash_tag(entry.hash);
    }
    for tag in tags.iter_mut().skip(len) {
        *tag = 0;
    }

    for b in 0..256 {
        let lo = radix.starts[b] as usize;
        let hi_exclusive = radix.starts[b + 1] as usize;
        if hi_exclusive <= lo || hi_exclusive - lo < RADIX_LINEAR_THRESHOLD {
            radix.interp_mul[b] = 0;
            continue;
        }
        let hi = hi_exclusive - 1;
        let lo_hash = index[lo].hash;
        let hi_hash = index[hi].hash;
        let span = hi_hash.wrapping_sub(lo_hash);
        radix.interp_mul[b] = interpolate_reciprocal(hi - lo, span);
    }
}

#[inline]
fn radix_tables(bytes: &[RadixLookupTables]) -> &RadixLookupTables {
    debug_assert!(!bytes.is_empty());
    &bytes[0]
}

#[doc(hidden)]
#[inline]
pub fn radix_tables_mut(bytes: &mut [RadixLookupTables]) -> &mut RadixLookupTables {
    debug_assert!(!bytes.is_empty());
    &mut bytes[0]
}

/// Precompute `floor(range * 2^64 / span)` for multiply/shift interpolation.
#[inline]
pub fn interpolate_reciprocal(range: usize, span: u64) -> u64 {
    if range == 0 || span == 0 {
        return 0;
    }
    (((range as u128) << 64) / span as u128) as u64
}

/// Decode an exact `[low, high]` window from precomputed top-byte partition starts.
fn top_byte_window<K, V>(
    _index: &[HashBackref<K, V>],
    radix: &RadixLookupTables,
    hash: u64,
    _steps: &mut SearchStepCounts,
) -> (usize, usize) {
    let bucket = (hash >> 56) as usize;
    let low = radix.starts[bucket] as usize;
    let hi_exclusive = radix.starts[bucket + 1] as usize;
    if hi_exclusive <= low {
        return (0, 0);
    }
    (low, hi_exclusive - 1)
}

#[inline]
fn interpolate_pos(
    lo: usize,
    hi: usize,
    lo_hash: u64,
    hi_hash: u64,
    hash: u64,
    recip: Option<u64>,
) -> usize {
    let span = hi_hash.wrapping_sub(lo_hash);
    if span == 0 {
        return lo;
    }

    let off = hash.wrapping_sub(lo_hash);
    let range = hi - lo;
    let mul = recip.unwrap_or_else(|| interpolate_reciprocal(range, span));
    let offset = (((off as u128) * mul as u128) >> 64) as usize;
    (lo + offset).clamp(lo, hi)
}

/// Scalar interpolation search for a hash in a sorted hash index.
///
/// Average-case complexity is `O(log log n)` when hashes are uniformly distributed.
pub fn interpolation_search<K, V>(index: &[HashBackref<K, V>], hash: u64) -> Option<usize> {
    if index.is_empty() {
        return None;
    }

    let mut lo = 0usize;
    let mut hi = index.len() - 1;

    while lo <= hi && hash >= index[lo].hash && hash <= index[hi].hash {
        if lo == hi {
            return (index[lo].hash == hash).then_some(lo);
        }

        let lo_hash = index[lo].hash;
        let hi_hash = index[hi].hash;
        let span = hi_hash.wrapping_sub(lo_hash);
        if span == 0 {
            for i in lo..=hi {
                if index[i].hash == hash {
                    return Some(i);
                }
            }
            return None;
        }

        let pos = interpolate_pos(lo, hi, lo_hash, hi_hash, hash, None);

        if index[pos].hash == hash {
            return Some(pos);
        }
        if index[pos].hash < hash {
            lo = pos + 1;
        } else if pos == 0 {
            return None;
        } else {
            hi = pos - 1;
        }
    }

    None
}

/// Hybrid lookup: coarse interpolation jumps, then SIMD tag filtering, then full-hash verification.
pub fn hybrid_interpolation_search<K, V>(
    index: &[HashBackref<K, V>],
    tags: &[u16],
    radix: Option<&RadixLookupTables>,
    hash: u64,
) -> Option<usize> {
    hybrid_interpolation_search_with_steps(index, tags, radix, hash).0
}

/// Hybrid lookup with per-phase step counts for analysis.
pub fn hybrid_interpolation_search_with_steps<K, V>(
    index: &[HashBackref<K, V>],
    tags: &[u16],
    radix: Option<&RadixLookupTables>,
    hash: u64,
) -> (Option<usize>, SearchStepCounts) {
    let mut steps = SearchStepCounts::default();
    let len = index.len();
    if len == 0 {
        return (None, steps);
    }

    let target_tag = hash_tag(hash);
    let bucket = (hash >> 56) as usize;
    let (mut low, mut high) = if let Some(radix) = radix {
        if radix.starts[bucket] == radix.starts[bucket + 1] {
            return (None, steps);
        }
        top_byte_window(index, radix, hash, &mut steps)
    } else {
        (0, len - 1)
    };

    if low > high || hash < index[low].hash || hash > index[high].hash {
        return (None, steps);
    }

    let window = high.saturating_sub(low) + 1;
    if window < RADIX_LINEAR_THRESHOLD {
        return (
            linear_tag_block_search(index, tags, hash, target_tag, low, high, len, &mut steps),
            steps,
        );
    }

    while high.saturating_sub(low) > TAG_BLOCK_SIZE {
        steps.coarse_steps += 1;
        let low_val = index[low].hash;
        let high_val = index[high].hash;

        if hash < low_val || hash > high_val {
            return (None, steps);
        }
        if low_val == high_val {
            break;
        }

        let recip = radix.and_then(|tables| {
            let bucket_lo = tables.starts[bucket] as usize;
            let bucket_hi = tables.starts[bucket + 1] as usize;
            (low == bucket_lo && high + 1 == bucket_hi && tables.interp_mul[bucket] != 0)
                .then_some(tables.interp_mul[bucket])
        });
        let pos = interpolate_pos(low, high, low_val, high_val, hash, recip);

        if index[pos].hash == hash {
            return (Some(pos), steps);
        }
        if index[pos].hash < hash {
            low = pos + 1;
        } else if pos == 0 {
            return (None, steps);
        } else {
            high = pos - 1;
        }
    }

    if hash < index[low].hash || hash > index[high].hash {
        return (None, steps);
    }

    let mut block_offset = low - (low % TAG_BLOCK_SIZE);
    while block_offset <= high {
        steps.simd_blocks += 1;
        if let Some(idx) = simd_verify_tag_block(
            index,
            tags,
            hash,
            target_tag,
            block_offset,
            low,
            high,
            len,
            &mut steps,
        ) {
            return (Some(idx), steps);
        }
        block_offset += TAG_BLOCK_SIZE;
    }

    (None, steps)
}

/// Walk `u16` tag blocks sequentially over a small radix window (no interpolation).
fn linear_tag_block_search<K, V>(
    index: &[HashBackref<K, V>],
    tags: &[u16],
    hash: u64,
    target_tag: u16,
    low: usize,
    high: usize,
    len: usize,
    steps: &mut SearchStepCounts,
) -> Option<usize> {
    let window = high.saturating_sub(low) + 1;
    steps.linear_tag_checks += window as u32;

    if window <= TAG_BLOCK_SIZE {
        return linear_tag_search_scalar(index, tags, hash, target_tag, low, high, steps);
    }

    let mut block_offset = low - (low % TAG_BLOCK_SIZE);
    while block_offset <= high {
        steps.simd_blocks += 1;
        if let Some(idx) = simd_verify_tag_block(
            index,
            tags,
            hash,
            target_tag,
            block_offset,
            low,
            high,
            len,
            steps,
        ) {
            return Some(idx);
        }
        block_offset += TAG_BLOCK_SIZE;
    }
    None
}

/// Scalar `u16` tag scan for very small windows that fit in one cache line.
#[inline]
fn linear_tag_search_scalar<K, V>(
    index: &[HashBackref<K, V>],
    tags: &[u16],
    hash: u64,
    target_tag: u16,
    low: usize,
    high: usize,
    steps: &mut SearchStepCounts,
) -> Option<usize> {
    for i in low..=high {
        if tags[i] == target_tag {
            steps.simd_candidates += 1;
            if index[i].hash == hash {
                return Some(i);
            }
        }
    }
    None
}

#[inline]
#[allow(unsafe_code)]
fn load_tag_block(tags: &[u16], block_offset: usize) -> u16x16 {
    debug_assert!(block_offset + TAG_BLOCK_SIZE <= tags.len());
    unsafe {
        let ptr = tags.as_ptr().add(block_offset) as *const [u16; TAG_BLOCK_SIZE];
        u16x16::new(*ptr)
    }
}

#[inline]
fn simd_verify_tag_block<K, V>(
    index: &[HashBackref<K, V>],
    tags: &[u16],
    hash: u64,
    target_tag: u16,
    block_offset: usize,
    low: usize,
    high: usize,
    len: usize,
    steps: &mut SearchStepCounts,
) -> Option<usize> {
    if block_offset + TAG_BLOCK_SIZE > tags.len() {
        return None;
    }

    let current_tags = load_tag_block(tags, block_offset);
    let target_vec = u16x16::splat(target_tag);
    let mut move_mask = current_tags.simd_eq(target_vec).to_bitmask();

    while move_mask != 0 {
        let lane = move_mask.trailing_zeros() as usize;
        let candidate_idx = block_offset + lane;

        if candidate_idx >= low && candidate_idx <= high && candidate_idx < len {
            steps.simd_candidates += 1;
            let candidate = &index[candidate_idx];
            if candidate.hash == hash {
                return Some(candidate_idx);
            }
        }

        move_mask &= move_mask - 1;
    }

    None
}

#[macro_export]
#[doc(hidden)]
macro_rules! __hash_sorted_map {
    (@gather $unique:ident $(#[$meta:meta])* $vis:vis static $name:ident: ($($map:tt)*) < $key:ty, $value:ty >;) => {
        // Type alias for element type projection.
        #[doc(hidden)]
        #[allow(unused, non_camel_case_types)]
        $vis type $name = $($map)* <$key, $value>;

        $(#[$meta])*
        $vis static $name: $($map)* <$key, $value> = {
            $crate::__support::link_section::declarative::section!(
                #[section(unsafe, type = typed, name = $name :: $unique :: RECORDS)]
                static RECORDS: $crate::__support::link_section::TypedSection<
                    $crate::hash_sorted_map::MapRecord<$key, $value>
                >;
            );

            $crate::__support::link_section::declarative::section!(
                #[section(unsafe, type = mutable, name = $name :: $unique :: INDEX)]
                static INDEX: $crate::__support::link_section::TypedMutableSection<
                    $crate::hash_sorted_map::HashBackref<$key, $value>
                >;
            );

            $crate::__support::link_section::declarative::section!(
                #[section(unsafe, type = mutable, name = $name :: $unique :: TAGS)]
                static TAGS: $crate::__support::link_section::TypedMutableSection<u16>;
            );

            $crate::__support::link_section::declarative::section!(
                #[section(unsafe, type = mutable, name = $name :: $unique :: TOP_BYTE)]
                static TOP_BYTE: $crate::__support::link_section::TypedMutableSection<
                    $crate::hash_sorted_map::RadixLookupTables
                >;
            );

            $crate::__support::link_section::declarative::in_section!(
                #[in_section(unsafe, type = mutable, name = $name :: $unique :: TAGS)]
                const _: [u16; $crate::hash_sorted_map::TAG_BLOCK_SIZE] =
                    $crate::hash_sorted_map::TAG_GATHER_PADDING;
            );

            $crate::__support::link_section::declarative::in_section!(
                #[in_section(unsafe, type = mutable, name = $name :: $unique :: TOP_BYTE)]
                const _: $crate::hash_sorted_map::RadixLookupTables =
                    $crate::hash_sorted_map::TOP_BYTE_INDEX_ZERO;
            );

            $crate::__support::ctor::declarative::ctor!(
                #[ctor(unsafe, anonymous, priority = 0)]
                fn __hash_sorted_map_init() {
                    unsafe {
                        $crate::hash_sorted_map::initialize_hash_sorted_map_index(
                            INDEX.as_mut_slice(),
                            TAGS.as_mut_slice(),
                            $crate::hash_sorted_map::radix_tables_mut(TOP_BYTE.as_mut_slice()),
                        );
                    }
                }
            );

            unsafe {
                $crate::hash_sorted_map::ScatteredHashSortedMap::new(
                    RECORDS.const_deref(),
                    INDEX.const_deref(),
                    TAGS.const_deref(),
                    TOP_BYTE.const_deref(),
                )
            }
        };
    };
    (@scatter [$collection_name:ident :: $unique:ident] ([$($meta:tt)*] => $(#[$imeta:meta])* $vis:vis $kind:ident $name:tt: $ty:ty = ($key_expr:expr, $value_expr:expr);)) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection_name :: $unique :: RECORDS, type = typed)]
            $(#[$imeta])*
            $vis $kind $name: <$($meta)* as $crate::ScatteredElementType>::T = $crate::hash_sorted_map::MapRecord::new(
                $key_expr,
                $value_expr,
            );
        );
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection_name :: $unique :: INDEX, type = mutable)]
            const _: $crate::hash_sorted_map::HashBackref<
                <$($meta)* as $crate::ScatteredElementTuple>::A,
                <$($meta)* as $crate::ScatteredElementTuple>::B,
            > = $crate::hash_sorted_map::HashBackref::new(
                $crate::const_hash!($key_expr),
                &$name as *const <$($meta)* as $crate::ScatteredElementType>::T,
            );
        );
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection_name :: $unique :: TAGS, type = mutable)]
            const _: u16 = $crate::hash_sorted_map::TAG_SCATTER_ZERO;
        );
    };
    (@scatter [$collection_name:ident :: $unique:ident] ([$($meta:tt)*] => $($rest:tt)*)) => {
        ::core::compile_error!(
            "invalid #[scatter] syntax for ScatteredHashSortedMap: expected \
             `static NAME: (Key, Value) = (<key>, <value>);` where the initializer \
             is a `(key, value)` tuple (for example: \
             `static FOO: (&'static str, u32) = (\"foo\", 1);`)"
        );
    };
}

#[cfg(all(test, not(miri)))]
mod link_tests {
    use super::{
        HashBackref, MapRecord, ScatteredHashSortedMap, SearchStepCounts, TAG_BLOCK_SIZE,
        TOP_BYTE_INDEX_ZERO, hash_tag, hybrid_interpolation_search,
        hybrid_interpolation_search_with_steps, initialize_hash_sorted_map_index,
        interpolation_search,
    };

    __hash_sorted_map!(@gather A pub static TEST_MAP: (ScatteredHashSortedMap)<&'static str, u32>;);
    __hash_sorted_map!(@scatter [TEST_MAP::A] ([TEST_MAP] => pub static APPLE: (&'static str, u32) = ("apple", 1);));
    __hash_sorted_map!(@scatter [TEST_MAP::A] ([TEST_MAP] => pub static BANANA: (&'static str, u32) = ("banana", 2);));

    #[test]
    fn scattered_hash_sorted_map_gather_scatter_find() {
        assert_eq!(TEST_MAP.len(), 2);
        assert_eq!(TEST_MAP.find(&"apple"), Some(&1));
        assert_eq!(TEST_MAP.find(&"banana"), Some(&2));
        assert_eq!(TEST_MAP.find(&"orange"), None);
        assert!(TEST_MAP.contains_key(&"apple"));

        let index = TEST_MAP.hash_index();
        assert!(index.windows(2).all(|w| w[0].hash <= w[1].hash));
        assert!(TEST_MAP.tags().len() >= TEST_MAP.len() + TAG_BLOCK_SIZE);
        assert_eq!(TEST_MAP.tags()[0], hash_tag(index[0].hash));
        assert_eq!(TEST_MAP.tags()[1], hash_tag(index[1].hash));
    }

    #[test]
    fn top_byte_bucket_sort_matches_full_sort() {
        use crate::hash::ConstHash;

        let records = [
            MapRecord::new("alpha", 1u32),
            MapRecord::new("beta", 2u32),
            MapRecord::new("gamma", 3u32),
            MapRecord::new("delta", 4u32),
            MapRecord::new("epsilon", 5u32),
        ];
        let make_index = || {
            [
                HashBackref::new(ConstHash::hash(&records[4].key), &records[4] as *const _),
                HashBackref::new(ConstHash::hash(&records[0].key), &records[0] as *const _),
                HashBackref::new(ConstHash::hash(&records[3].key), &records[3] as *const _),
                HashBackref::new(ConstHash::hash(&records[1].key), &records[1] as *const _),
                HashBackref::new(ConstHash::hash(&records[2].key), &records[2] as *const _),
            ]
        };
        let mut bucket_sorted = make_index();
        let mut full_sorted = make_index();
        let mut radix = TOP_BYTE_INDEX_ZERO;
        let mut tags = [0_u16; 5 + TAG_BLOCK_SIZE];
        let mut scratch = Vec::new();

        super::sort_index_by_top_byte_bucket(&mut bucket_sorted, &mut radix, &mut scratch);
        full_sorted.sort_unstable_by_key(|entry| entry.hash);

        for (left, right) in bucket_sorted.iter().zip(full_sorted.iter()) {
            assert_eq!(left.hash, right.hash);
            assert_eq!(left.record, right.record);
        }

        super::finish_sorted_index(&bucket_sorted, &mut tags, &mut radix);
        for record in &records {
            let hash = ConstHash::hash(&record.key);
            let idx = hybrid_interpolation_search(&bucket_sorted, &tags, Some(&radix), hash)
                .expect("sorted bucket map should find hash");
            assert_eq!(unsafe { &*bucket_sorted[idx].record }.key, record.key);
        }
    }

    #[test]
    fn interpolation_search_unit() {
        let records = [
            MapRecord::new("a", 1u32),
            MapRecord::new("b", 2u32),
            MapRecord::new("c", 3u32),
        ];
        let mut index = [
            HashBackref::new(crate::const_hash!("c"), &records[2] as *const _),
            HashBackref::new(crate::const_hash!("a"), &records[0] as *const _),
            HashBackref::new(crate::const_hash!("b"), &records[1] as *const _),
        ];
        let mut tags = [0_u16; 3 + TAG_BLOCK_SIZE];
        let mut radix = TOP_BYTE_INDEX_ZERO;
        initialize_hash_sorted_map_index(&mut index, &mut tags, &mut radix);

        for key in ["a", "b", "c"] {
            let hash = crate::const_hash!(key);
            let idx = interpolation_search(&index, hash).expect("hash should be found");
            assert_eq!(index[idx].hash, hash);
            assert_eq!(unsafe { &*index[idx].record }.key, key);

            let hybrid_idx = hybrid_interpolation_search(&index, &tags, Some(&radix), hash)
                .expect("hybrid should find hash");
            assert_eq!(hybrid_idx, idx);
        }
        assert_eq!(interpolation_search(&index, crate::const_hash!("z")), None);
        assert_eq!(
            hybrid_interpolation_search(&index, &tags, Some(&radix), crate::const_hash!("z")),
            None
        );
    }

    #[test]
    fn search_step_distribution_on_bench_keys() {
        use crate::hash::ConstHash;

        const NUM_RECORDS: usize = 5000;
        let strings: Vec<&'static str> = (0..NUM_RECORDS)
            .map(|i| Box::leak(format!("key{i:04}").into_boxed_str()) as &'static str)
            .collect();
        let records: Vec<MapRecord<&'static str, u32>> = strings
            .iter()
            .enumerate()
            .map(|(i, s)| MapRecord::new(*s, i as u32))
            .collect();
        let mut index: Vec<HashBackref<&'static str, u32>> = records
            .iter()
            .map(|record| {
                HashBackref::new(
                    ConstHash::hash(&record.key),
                    record as *const _ as *const _,
                )
            })
            .collect();
        let mut tags = vec![0_u16; NUM_RECORDS + TAG_BLOCK_SIZE];
        let mut radix = TOP_BYTE_INDEX_ZERO;
        initialize_hash_sorted_map_index(&mut index, &mut tags, &mut radix);

        let mut hybrid_totals = SearchStepCounts::default();
        let mut hybrid_no_top = SearchStepCounts::default();
        let mut scalar_steps = 0u64;
        let mut window_sizes = Vec::new();

        for record in &records {
            let hash = ConstHash::hash(&record.key);
            let (result, steps) =
                hybrid_interpolation_search_with_steps(&index, &tags, Some(&radix), hash);
            assert_eq!(result.is_some(), true);
            hybrid_totals.coarse_steps += steps.coarse_steps;
            hybrid_totals.top_byte_expand_steps += steps.top_byte_expand_steps;
            hybrid_totals.linear_tag_checks += steps.linear_tag_checks;
            hybrid_totals.simd_blocks += steps.simd_blocks;
            hybrid_totals.simd_candidates += steps.simd_candidates;

            let (_, steps_no_top) =
                hybrid_interpolation_search_with_steps(&index, &tags, None, hash);
            hybrid_no_top.coarse_steps += steps_no_top.coarse_steps;
            hybrid_no_top.simd_blocks += steps_no_top.simd_blocks;
            hybrid_no_top.simd_candidates += steps_no_top.simd_candidates;

            let mut lo = 0usize;
            let mut hi = index.len() - 1;
            let mut scalar = 0u32;
            while lo <= hi && hash >= index[lo].hash && hash <= index[hi].hash {
                scalar += 1;
                if lo == hi {
                    break;
                }
                let lo_hash = index[lo].hash;
                let hi_hash = index[hi].hash;
                let span = hi_hash.wrapping_sub(lo_hash);
                if span == 0 {
                    break;
                }
                let pos = super::interpolate_pos(lo, hi, lo_hash, hi_hash, hash, None);
                if index[pos].hash == hash {
                    break;
                }
                if index[pos].hash < hash {
                    lo = pos + 1;
                } else if pos == 0 {
                    break;
                } else {
                    hi = pos - 1;
                }
            }
            scalar_steps += scalar as u64;

            let bucket = (hash >> 56) as usize;
            let low = radix.starts[bucket] as usize;
            let high = radix.starts[bucket + 1] as usize;
            if high > low {
                window_sizes.push(high - low);
            }
        }

        let n = records.len() as f64;
        eprintln!("=== hash-sorted map search step analysis (n={NUM_RECORDS}) ===");
        eprintln!(
            "hybrid + top-byte: coarse={:.2} expand={:.2} linear_tags={:.2} simd_blocks={:.2} simd_candidates={:.2}",
            hybrid_totals.coarse_steps as f64 / n,
            hybrid_totals.top_byte_expand_steps as f64 / n,
            hybrid_totals.linear_tag_checks as f64 / n,
            hybrid_totals.simd_blocks as f64 / n,
            hybrid_totals.simd_candidates as f64 / n,
        );
        eprintln!(
            "hybrid no top-byte: coarse={:.2} simd_blocks={:.2} simd_candidates={:.2}",
            hybrid_no_top.coarse_steps as f64 / n,
            hybrid_no_top.simd_blocks as f64 / n,
            hybrid_no_top.simd_candidates as f64 / n,
        );
        eprintln!(
            "scalar interpolation only: steps={:.2}",
            scalar_steps as f64 / n,
        );
        window_sizes.sort_unstable();
        let p50 = window_sizes[window_sizes.len() / 2];
        let p95 = window_sizes[window_sizes.len() * 95 / 100];
        let avg_window = window_sizes.iter().sum::<usize>() as f64 / n;
        eprintln!(
            "top-byte initial window: avg={avg_window:.1} p50={p50} p95={p95} max={}",
            window_sizes.last().copied().unwrap_or(0)
        );
    }

    __hash_sorted_map!(@gather B pub static EMPTY_MAP: (ScatteredHashSortedMap)<&'static str, u32>;);

    #[test]
    fn test_empty_scattered_hash_sorted_map() {
        assert_eq!(EMPTY_MAP.len(), 0);
        assert!(EMPTY_MAP.is_empty());
        assert_eq!(EMPTY_MAP.find(&"apple"), None);
        assert!(!EMPTY_MAP.contains_key(&"apple"));
        assert!(EMPTY_MAP.tags().len() >= TAG_BLOCK_SIZE);
    }
}
