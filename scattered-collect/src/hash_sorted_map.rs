//! A hash-sorted map initialized with link-time data.
#![doc = concat!("```rust\n", include_str!("../examples/hash_sorted_map.rs"), "\n```\n")]

use link_section::{TypedMutableSection, TypedSection};

use crate::hash::ConstHash;

// Tag width and SIMD lane count are selected by these two aliases; everything
// below derives from them. Alternatives: `(u16, u16x16)` or `(u8, u8x16)` for 16 lanes.
/// Scalar type of a SIMD tag: the low bits of a hash used for filtering.
pub type Tag = u8;

/// SIMD vector of [`Tag`]s compared in one step.
type TagVector = wide::u8x32;

/// The number of [`Tag`]s compared per SIMD step — the lane count of [`TagVector`].
pub const TAG_BLOCK_SIZE: usize = size_of::<TagVector>() / size_of::<Tag>();

/// Below this radix-bucket width, skip interpolation and linearly scan tags.
pub const RADIX_LINEAR_THRESHOLD: usize = 64;

/// Zero [`Tag`] slots reserved at gather time so the final [`TAG_BLOCK_SIZE`]-wide
/// SIMD window can load past the last record without branching.
#[doc(hidden)]
pub const TAG_GATHER_PADDING: [Tag; TAG_BLOCK_SIZE] = [0; TAG_BLOCK_SIZE];

/// Per-scatter tag placeholder written before init fills the real tag.
#[doc(hidden)]
pub const TAG_SCATTER_ZERO: Tag = 0;

/// Maximum top hash bits used to partition the index. Init uses up to this many (fewer
/// for small maps), keeping buckets ≈ `len / 2^RADIX_BITS` so lookups stay in the linear
/// SIMD path as the map grows.
pub const RADIX_BITS: usize = 12;

/// Number of radix partition slots: one per bucket plus a trailing `starts[n] == len`.
pub const RADIX_SLOTS: usize = (1 << RADIX_BITS) + 1;

/// Precomputed radix metadata: exact bucket starts plus the shift selecting the bucket.
#[repr(C)]
pub struct RadixLookupTables {
    /// Exact partition start for each bucket; `starts[bucket_count] == len`.
    pub starts: [u16; RADIX_SLOTS],
    /// Right-shift applied to a hash to select its bucket (`64 - dispatch_bits`).
    pub shift: u8,
}

impl RadixLookupTables {
    /// Zero-filled tables written at gather and populated during initialization.
    pub const ZERO: Self = Self {
        starts: [0; RADIX_SLOTS],
        shift: 0,
    };
}

/// The radix partition starts array.
pub type RadixStarts = [u16; RADIX_SLOTS];

/// Zero-filled radix tables written at gather.
#[doc(hidden)]
pub const RADIX_TABLES_ZERO: RadixLookupTables = RadixLookupTables::ZERO;

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
#[allow(unsafe_code)]
unsafe impl<K: Sync, V: Sync> Sync for HashBackref<K, V> {}
#[allow(unsafe_code)]
unsafe impl<K: Send, V: Send> Send for HashBackref<K, V> {}

/// A map whose hash index is sorted at link time for hybrid interpolation + SIMD lookup.
///
/// Scatter sites contribute a [`MapRecord`], a matching [`HashBackref`], and a zero
/// [`Tag`] placeholder. A priority-0 constructor sorts the hash index and writes
/// the low [`Tag`] bits of each sorted hash into the tag section.
///
/// For swiss-table lookup, use [`crate::ScatteredMap`]. For a sorted slice without
/// key lookup, use [`crate::ScatteredSortedSlice`].
/// ```rust
#[doc = include_str!("../examples/hash_sorted_map.rs")]
/// ```
pub struct ScatteredHashSortedMap<K: 'static, V: 'static> {
    records: &'static TypedSection<MapRecord<K, V>>,
    index: &'static TypedMutableSection<HashBackref<K, V>>,
    tags: &'static TypedMutableSection<Tag>,
    radix: &'static TypedMutableSection<RadixLookupTables>,
}

impl<K: 'static, V: 'static> ScatteredHashSortedMap<K, V> {
    #[doc(hidden)]
    #[allow(unsafe_code)]
    pub const unsafe fn new(
        records: &'static TypedSection<MapRecord<K, V>>,
        index: &'static TypedMutableSection<HashBackref<K, V>>,
        tags: &'static TypedMutableSection<Tag>,
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
    #[allow(unsafe_code)]
    pub fn find(&self, key: &K) -> Option<&V>
    where
        K: ConstHash + PartialEq,
    {
        let hash = ConstHash::hash(key);
        // `first()` yields `None` for an empty radix section (e.g. an empty map under
        // Miri, where link sections don't populate), so an empty map returns `None`.
        let radix = self.radix.as_slice().first()?;
        let idx = hybrid_interpolation_search(
            self.index.as_slice(),
            self.tags.as_slice(),
            Some(radix),
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

    /// The SIMD tag slice. The first [`Self::len`] entries hold the low [`Tag`] bits
    /// of each sorted hash; trailing entries are zero padding for safe SIMD loads.
    #[inline]
    pub fn tags(&self) -> &[Tag] {
        self.tags.as_slice()
    }

    /// Precomputed radix partition starts and bucket shift.
    #[inline]
    pub fn radix_tables(&self) -> &RadixLookupTables {
        radix_tables(self.radix.as_slice())
    }

    /// Exact radix partition starts (`starts[bucket_count] == len`).
    #[inline]
    pub fn radix_starts(&self) -> &RadixStarts {
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
    #[allow(unsafe_code)]
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

/// The low [`Tag`] bits of a hash, used for SIMD tag filtering.
#[inline]
pub(crate) const fn hash_tag(hash: u64) -> Tag {
    hash as Tag
}

/// Per-phase step counts for search analysis.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SearchStepCounts {
    /// Coarse interpolation jumps before the SIMD phase.
    pub coarse_steps: u32,
    /// Top-byte window expansion steps after the first jump.
    pub top_byte_expand_steps: u32,
    /// SIMD tag blocks examined ([`TAG_BLOCK_SIZE`] lanes each).
    pub simd_blocks: u32,
    /// SIMD tag matches that required a full-hash check.
    pub simd_candidates: u32,
    /// Scalar tag comparisons in a small radix bucket.
    pub linear_tag_checks: u32,
}

/// Sort the hash index and write sorted SIMD tags and the top-byte jump table.
pub fn initialize_hash_sorted_map_index<K, V>(
    index: &mut [HashBackref<K, V>],
    tags: &mut [Tag],
    radix: &mut RadixLookupTables,
) {
    let mut scratch = ::std::vec::Vec::new();
    let mut counts = ::std::vec::Vec::new();
    initialize_hash_sorted_map_index_with_scratch(index, tags, radix, &mut scratch, &mut counts);
}

/// Like [`initialize_hash_sorted_map_index`], but reuses the `scratch` and `counts`
/// buffers across calls (the gather constructor allocates them fresh per map).
pub(crate) fn initialize_hash_sorted_map_index_with_scratch<K, V>(
    index: &mut [HashBackref<K, V>],
    tags: &mut [Tag],
    radix: &mut RadixLookupTables,
    scratch: &mut ::std::vec::Vec<HashBackref<K, V>>,
    counts: &mut ::std::vec::Vec<u32>,
) {
    let len = index.len();
    if len == 0 {
        *radix = RadixLookupTables::ZERO;
        return;
    }
    if len > tags.len() {
        panic!("tag section must have at least as many entries as the hash index (need {len})");
    }

    sort_index_by_predicted_position(index, radix, scratch, counts);
    finish_sorted_index(index, tags);
}

/// Distribution sort for uniform hashes: a counting sort on the top `bits` of the hash
/// (≈ one bucket per record), then one insertion-sort cleanup. Each record lands within
/// a bucket of its final slot, so the whole sort is O(n) with no per-bucket comparison
/// sort. `counts` is a reusable bucket-offset buffer.
#[allow(unsafe_code)]
fn sort_index_by_predicted_position<K, V>(
    index: &mut [HashBackref<K, V>],
    radix: &mut RadixLookupTables,
    scratch: &mut ::std::vec::Vec<HashBackref<K, V>>,
    counts: &mut ::std::vec::Vec<u32>,
) {
    let len = index.len();
    if len > u16::MAX as usize {
        panic!(
            "ScatteredHashSortedMap supports at most {} records",
            u16::MAX
        );
    }

    // ≈ one fine bucket per record: smallest `2^bits >= len`, clamped so buckets still
    // nest inside the dispatch radix (>= 8) and the count array stays bounded (<= 1 << 16).
    let bits = (usize::BITS - (len.max(1) - 1).leading_zeros()).clamp(8, 16);
    let num_buckets = 1usize << bits;
    let fine_shift = 64 - bits;

    counts.clear();
    counts.resize(num_buckets + 1, 0);
    for entry in index.iter() {
        counts[(entry.hash >> fine_shift) as usize] += 1;
    }

    // Exclusive prefix sum: `counts[b]` becomes the start offset of fine bucket `b`.
    let mut acc = 0u32;
    for slot in counts.iter_mut() {
        let next = acc + *slot;
        *slot = acc;
        acc = next;
    }

    // Read the dispatch table off the fine prefix sums, capped at `RADIX_BITS` so the
    // lookup table stays a fixed `RADIX_SLOTS`. Dispatch bucket `b` starts at fine bucket
    // `b << (bits - dispatch_bits)`.
    let dispatch_bits = bits.min(RADIX_BITS as u32);
    let dispatch_buckets = 1usize << dispatch_bits;
    let fine_per_dispatch = 1usize << (bits - dispatch_bits);
    for (b, start) in radix.starts.iter_mut().enumerate().take(dispatch_buckets) {
        *start = counts[b * fine_per_dispatch] as u16;
    }
    radix.starts[dispatch_buckets] = len as u16;
    radix.shift = (64 - dispatch_bits) as u8;

    // Scatter into fine buckets (within a bucket, entries keep scatter order).
    scratch.clear();
    scratch.reserve(len);
    unsafe {
        ::core::ptr::copy_nonoverlapping(index.as_ptr(), scratch.as_mut_ptr(), len);
        scratch.set_len(len);
    }
    for entry in scratch.iter() {
        let bucket = (entry.hash >> fine_shift) as usize;
        let dest = counts[bucket] as usize;
        unsafe {
            ::core::ptr::write(
                index.get_unchecked_mut(dest),
                ::core::ptr::read(entry as *const HashBackref<K, V>),
            );
        }
        counts[bucket] += 1;
    }

    // Cleanup: only entries sharing a fine bucket can be out of order, and they sit
    // adjacent, so each entry moves O(bucket size) ≈ O(1).
    for i in 1..len {
        let mut j = i;
        while j > 0 && index[j - 1].hash > index[j].hash {
            index.swap(j - 1, j);
            j -= 1;
        }
    }
}

/// Write the SIMD tags (low [`Tag`] bits of each sorted hash) plus zero padding.
fn finish_sorted_index<K, V>(index: &[HashBackref<K, V>], tags: &mut [Tag]) {
    let len = index.len();
    for (tag, entry) in tags.iter_mut().zip(index.iter()).take(len) {
        *tag = hash_tag(entry.hash);
    }
    for tag in tags.iter_mut().skip(len) {
        *tag = 0;
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
pub(crate) fn interpolate_reciprocal(range: usize, span: u64) -> u64 {
    if range == 0 || span == 0 {
        return 0;
    }
    (((range as u128) << 64) / span as u128) as u64
}

/// Decode the `[low, high]` window for `hash` from the radix partition starts, or
/// `None` if its bucket is empty.
fn radix_window(radix: &RadixLookupTables, hash: u64) -> Option<(usize, usize)> {
    let bucket = (hash >> radix.shift) as usize;
    let low = radix.starts[bucket] as usize;
    let hi_exclusive = radix.starts[bucket + 1] as usize;
    (hi_exclusive > low).then(|| (low, hi_exclusive - 1))
}

#[inline]
fn interpolate_pos(lo: usize, hi: usize, lo_hash: u64, hi_hash: u64, hash: u64) -> usize {
    let span = hi_hash.wrapping_sub(lo_hash);
    if span == 0 {
        return lo;
    }

    let off = hash.wrapping_sub(lo_hash);
    let range = hi - lo;
    let mul = interpolate_reciprocal(range, span);
    let offset = (((off as u128) * mul as u128) >> 64) as usize;
    (lo + offset).clamp(lo, hi)
}

/// Scalar interpolation search for a hash in a sorted hash index. Used as a baseline
/// in tests and benchmarks; the production lookup uses [`hybrid_interpolation_search`].
///
/// Average-case complexity is `O(log log n)` when hashes are uniformly distributed.
#[cfg(any(test, feature = "__internal"))]
pub(crate) fn interpolation_search<K, V>(index: &[HashBackref<K, V>], hash: u64) -> Option<usize> {
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
            return index[lo..=hi]
                .iter()
                .position(|entry| entry.hash == hash)
                .map(|offset| lo + offset);
        }

        let pos = interpolate_pos(lo, hi, lo_hash, hi_hash, hash);

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
pub(crate) fn hybrid_interpolation_search<K, V>(
    index: &[HashBackref<K, V>],
    tags: &[Tag],
    radix: Option<&RadixLookupTables>,
    hash: u64,
) -> Option<usize> {
    hybrid_interpolation_search_with_steps(index, tags, radix, hash).0
}

/// Hybrid lookup with per-phase step counts for analysis.
pub(crate) fn hybrid_interpolation_search_with_steps<K, V>(
    index: &[HashBackref<K, V>],
    tags: &[Tag],
    radix: Option<&RadixLookupTables>,
    hash: u64,
) -> (Option<usize>, SearchStepCounts) {
    let mut steps = SearchStepCounts::default();
    let len = index.len();
    if len == 0 {
        return (None, steps);
    }

    let (mut low, mut high) = match radix {
        Some(radix) => match radix_window(radix, hash) {
            Some(window) => window,
            None => return (None, steps),
        },
        None => (0, len - 1),
    };

    if low > high || hash < index[low].hash || hash > index[high].hash {
        return (None, steps);
    }

    let window = high.saturating_sub(low) + 1;
    if window < RADIX_LINEAR_THRESHOLD {
        return (
            linear_tag_block_search(index, tags, hash, low, high, &mut steps),
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

        let pos = interpolate_pos(low, high, low_val, high_val, hash);

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

    // Start at `low` (unaligned); aligning down to a block boundary only wastes
    // an extra block when `low` sits late in its lane group.
    let mut block_offset = low;
    while block_offset <= high {
        steps.simd_blocks += 1;
        if let Some(idx) =
            simd_verify_tag_block(index, tags, hash, block_offset, low, high, &mut steps)
        {
            return (Some(idx), steps);
        }
        block_offset += TAG_BLOCK_SIZE;
    }

    (None, steps)
}

/// Walk [`Tag`] blocks sequentially over a small radix window (no interpolation).
fn linear_tag_block_search<K, V>(
    index: &[HashBackref<K, V>],
    tags: &[Tag],
    hash: u64,
    low: usize,
    high: usize,
    steps: &mut SearchStepCounts,
) -> Option<usize> {
    let window = high.saturating_sub(low) + 1;
    steps.linear_tag_checks += window as u32;

    let mut block_offset = low;
    while block_offset <= high {
        steps.simd_blocks += 1;
        if let Some(idx) = simd_verify_tag_block(index, tags, hash, block_offset, low, high, steps)
        {
            return Some(idx);
        }
        block_offset += TAG_BLOCK_SIZE;
    }
    None
}

#[inline]
#[allow(unsafe_code)]
fn load_tag_block(tags: &[Tag], block_offset: usize) -> TagVector {
    debug_assert!(block_offset + TAG_BLOCK_SIZE <= tags.len());
    unsafe {
        let ptr = tags.as_ptr().add(block_offset) as *const [Tag; TAG_BLOCK_SIZE];
        TagVector::new(*ptr)
    }
}

#[inline]
fn simd_verify_tag_block<K, V>(
    index: &[HashBackref<K, V>],
    tags: &[Tag],
    hash: u64,
    block_offset: usize,
    low: usize,
    high: usize,
    steps: &mut SearchStepCounts,
) -> Option<usize> {
    if block_offset + TAG_BLOCK_SIZE > tags.len() {
        return None;
    }

    let current_tags = load_tag_block(tags, block_offset);
    let target_vec = TagVector::splat(hash_tag(hash));
    let mut move_mask = current_tags.simd_eq(target_vec).to_bitmask();

    while move_mask != 0 {
        let lane = move_mask.trailing_zeros() as usize;
        let candidate_idx = block_offset + lane;

        if candidate_idx >= low && candidate_idx <= high && candidate_idx < index.len() {
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

/// Search and init primitives exposed to benchmarks under the `__internal` feature.
#[cfg(feature = "__internal")]
pub mod internal {
    /// See [`super::hybrid_interpolation_search`].
    pub fn hybrid_interpolation_search<K, V>(
        index: &[super::HashBackref<K, V>],
        tags: &[super::Tag],
        radix: Option<&super::RadixLookupTables>,
        hash: u64,
    ) -> Option<usize> {
        super::hybrid_interpolation_search(index, tags, radix, hash)
    }

    /// See [`super::interpolation_search`].
    pub fn interpolation_search<K, V>(
        index: &[super::HashBackref<K, V>],
        hash: u64,
    ) -> Option<usize> {
        super::interpolation_search(index, hash)
    }

    /// See [`super::initialize_hash_sorted_map_index_with_scratch`].
    pub fn initialize_hash_sorted_map_index_with_scratch<K, V>(
        index: &mut [super::HashBackref<K, V>],
        tags: &mut [super::Tag],
        radix: &mut super::RadixLookupTables,
        scratch: &mut ::std::vec::Vec<super::HashBackref<K, V>>,
        counts: &mut ::std::vec::Vec<u32>,
    ) {
        super::initialize_hash_sorted_map_index_with_scratch(index, tags, radix, scratch, counts);
    }
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
                static TAGS: $crate::__support::link_section::TypedMutableSection<
                    $crate::hash_sorted_map::Tag
                >;
            );

            $crate::__support::link_section::declarative::section!(
                #[section(unsafe, type = mutable, name = $name :: $unique :: TOP_BYTE)]
                static TOP_BYTE: $crate::__support::link_section::TypedMutableSection<
                    $crate::hash_sorted_map::RadixLookupTables
                >;
            );

            $crate::__support::link_section::declarative::in_section!(
                #[in_section(unsafe, type = mutable, name = $name :: $unique :: TAGS)]
                const _: [$crate::hash_sorted_map::Tag; $crate::hash_sorted_map::TAG_BLOCK_SIZE] =
                    $crate::hash_sorted_map::TAG_GATHER_PADDING;
            );

            $crate::__support::link_section::declarative::in_section!(
                #[in_section(unsafe, type = mutable, name = $name :: $unique :: TOP_BYTE)]
                const _: $crate::hash_sorted_map::RadixLookupTables =
                    $crate::hash_sorted_map::RADIX_TABLES_ZERO;
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
            const _: $crate::hash_sorted_map::Tag = $crate::hash_sorted_map::TAG_SCATTER_ZERO;
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
        HashBackref, MapRecord, RADIX_TABLES_ZERO, ScatteredHashSortedMap, SearchStepCounts,
        TAG_BLOCK_SIZE, Tag, hash_tag, hybrid_interpolation_search,
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

    /// Small input (below 256 records, so `bits` clamps to 8) — the distribution sort
    /// still matches a full comparison sort and drives correct lookups.
    #[test]
    fn distribution_sort_small_input_matches_full_sort() {
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
        let mut dist_sorted = make_index();
        let mut full_sorted = make_index();
        let mut radix = RADIX_TABLES_ZERO;
        let mut tags = [0 as Tag; 5 + TAG_BLOCK_SIZE];
        let mut scratch = Vec::new();
        let mut counts = Vec::new();

        super::sort_index_by_predicted_position(
            &mut dist_sorted,
            &mut radix,
            &mut scratch,
            &mut counts,
        );
        full_sorted.sort_unstable_by_key(|entry| entry.hash);

        for (left, right) in dist_sorted.iter().zip(full_sorted.iter()) {
            assert_eq!(left.hash, right.hash);
            assert_eq!(left.record, right.record);
        }

        super::finish_sorted_index(&dist_sorted, &mut tags);
        for record in &records {
            let hash = ConstHash::hash(&record.key);
            let idx = hybrid_interpolation_search(&dist_sorted, &tags, Some(&radix), hash)
                .expect("sorted map should find hash");
            assert_eq!(unsafe { &*dist_sorted[idx].record }.key, record.key);
        }
    }

    #[test]
    fn distribution_sort_matches_full_sort() {
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
        let make_index = || -> Vec<HashBackref<&'static str, u32>> {
            records
                .iter()
                .map(|record| {
                    HashBackref::new(ConstHash::hash(&record.key), record as *const _ as *const _)
                })
                .collect()
        };

        let mut dist_sorted = make_index();
        let mut full_sorted = make_index();
        let mut radix = RADIX_TABLES_ZERO;
        let mut scratch = Vec::new();
        let mut counts = Vec::new();

        super::sort_index_by_predicted_position(
            &mut dist_sorted,
            &mut radix,
            &mut scratch,
            &mut counts,
        );
        full_sorted.sort_by_key(|entry| entry.hash);

        // Fully sorted by hash, identical to a comparison sort.
        for (left, right) in dist_sorted.iter().zip(full_sorted.iter()) {
            assert_eq!(left.hash, right.hash);
        }

        // The derived top-byte dispatch table drives correct lookups.
        let mut tags = vec![0 as Tag; NUM_RECORDS + TAG_BLOCK_SIZE];
        super::finish_sorted_index(&dist_sorted, &mut tags);
        for record in &records {
            let hash = ConstHash::hash(&record.key);
            let idx = hybrid_interpolation_search(&dist_sorted, &tags, Some(&radix), hash)
                .expect("dist-sorted map should find hash");
            assert_eq!(unsafe { &*dist_sorted[idx].record }.key, record.key);
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
        let mut tags = [0 as Tag; 3 + TAG_BLOCK_SIZE];
        let mut radix = RADIX_TABLES_ZERO;
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
                HashBackref::new(ConstHash::hash(&record.key), record as *const _ as *const _)
            })
            .collect();
        let mut tags = vec![0 as Tag; NUM_RECORDS + TAG_BLOCK_SIZE];
        let mut radix = RADIX_TABLES_ZERO;
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
                let pos = super::interpolate_pos(lo, hi, lo_hash, hi_hash, hash);
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

            let bucket = (hash >> radix.shift) as usize;
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
