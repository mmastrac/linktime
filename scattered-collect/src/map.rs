use core::marker::PhantomData;
use core::mem::{align_of, size_of};

use link_section::TypedSection;
use wide::u8x16;

/// One SIMD group: 16 control bytes (`0` = empty; occupied uses **7-bit fingerprint** in the low
/// bits with **bit `0x80` set** so a used slot is never byte `0`) plus parallel hash / row index
/// arrays.
#[repr(C)]
pub struct MetadataSlice {
    pub buckets: wide::u8x16,
    pub hashes: [u64; 16],
    pub indexes: [u32; 16],
}

/// One gathered map entry.
///
/// Scatter sites store the hash next to the key and value so map initialization
/// only needs to place rows into metadata slots.
#[repr(C)]
pub struct MapRecord<K, V> {
    pub key: K,
    pub value: V,
    pub hash: u64,
}

impl<K, V> MapRecord<K, V> {
    pub const fn new(key: K, value: V, hash: u64) -> Self {
        Self { key, value, hash }
    }
}

/// How many SIMD groups (each 16 slots) are needed for `n` inserts at ~70% load.
pub const fn num_groups_for_records(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let slots = (n as u128 * 10 / 7) as usize + 1;
    let groups = (slots + 15) / 16;
    groups
}

/// Bytes required for `refs` passed to [`initialize_scattered_map`], including alignment padding
/// before the first [`MetadataSlice`].
pub const fn scattered_map_refs_min_bytes(num_groups: usize) -> usize {
    if num_groups == 0 {
        return 0;
    }
    (align_of::<MetadataSlice>() - 1) + num_groups * size_of::<MetadataSlice>()
}

/// Conservative metadata bytes reserved in each map's aux section at gather time.
pub const fn scattered_map_metadata_reserve_bytes() -> usize {
    scattered_map_refs_min_bytes(num_groups_for_records(256))
}

/// Pluggable probe strategy for the scattered map.
///
/// For [`LinearProbe`], `len` is the number of **groups** (each group is 16 slots). The iterator
/// must visit every group index `0..len` exactly once per full cycle (wrap with `% len`).
pub trait ProbeStrategy {
    fn new(len: usize, hash: u64) -> Self;
    fn next(&mut self) -> Option<usize>;
}

/// Linear probing over **groups**: start at `((hash >> 7) % len)`, then `(i+1) % len`, …
#[derive(Debug, Clone, Copy, Default)]
pub struct LinearProbe {
    len: usize,
    remaining: usize,
    pos: usize,
}

impl ProbeStrategy for LinearProbe {
    fn new(len: usize, hash: u64) -> Self {
        debug_assert!(len > 0);
        let pos = ((hash >> 7) as usize) % len;
        Self {
            len,
            remaining: len,
            pos,
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<usize> {
        if self.remaining == 0 {
            return None;
        }
        let p = self.pos;
        self.pos = (self.pos + 1) % self.len;
        self.remaining -= 1;
        Some(p)
    }
}

pub trait ConstHash {
    type Hasher;
    fn hash(&self) -> u64;
}

pub struct ConstHasher<T> {
    _phantom: core::marker::PhantomData<T>,
}

impl ConstHash for &'static str {
    type Hasher = ConstHasher<&'static str>;
    fn hash(&self) -> u64 {
        xxhash_rust::xxh3::xxh3_64(self.as_bytes())
    }
}

impl ConstHasher<&'static str> {
    pub fn const_hash(s: &'static str) -> u64 {
        xxhash_rust::const_xxh3::xxh3_64(s.as_bytes())
    }
}

/// Mutable state filled by the map initialization constructor.
pub struct ScatteredMapState<K, V, P: ProbeStrategy = LinearProbe> {
    table: core::cell::UnsafeCell<Option<ScatteredMapTable<K, V, P>>>,
}

unsafe impl<K: Sync, V: Sync, P: ProbeStrategy> Sync for ScatteredMapState<K, V, P> {}

impl<K, V, P: ProbeStrategy> ScatteredMapState<K, V, P> {
    pub const fn new() -> Self {
        Self {
            table: core::cell::UnsafeCell::new(None),
        }
    }

    /// # Safety
    ///
    /// This must be called at most once, before concurrent access to the map
    /// can happen.
    pub unsafe fn initialize(&self, table: ScatteredMapTable<K, V, P>) {
        unsafe {
            *self.table.get() = Some(table);
        }
    }

    fn table(&self) -> &ScatteredMapTable<K, V, P> {
        unsafe {
            (*self.table.get())
                .as_ref()
                .expect("ScatteredMap used before its initialization constructor ran")
        }
    }
}

impl<K, V, P: ProbeStrategy> Default for ScatteredMapState<K, V, P> {
    fn default() -> Self {
        Self::new()
    }
}

/// User-facing scattered map wrapper.
///
/// The gathered records are kept in arbitrary link order; lookup uses metadata
/// prepared by a priority-0 constructor.
pub struct ScatteredMap<K: 'static, V: 'static, P: ProbeStrategy + 'static = LinearProbe> {
    section: &'static TypedSection<MapRecord<K, V>>,
    state: &'static ScatteredMapState<K, V, P>,
}

impl<K: 'static, V: 'static, P: ProbeStrategy + 'static> ScatteredMap<K, V, P> {
    #[doc(hidden)]
    #[allow(unsafe_code)]
    pub const unsafe fn new(
        section: &'static TypedSection<MapRecord<K, V>>,
        state: &'static ScatteredMapState<K, V, P>,
    ) -> Self {
        Self { section, state }
    }

    /// The number of records in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.section.len()
    }

    /// True if the map has no records.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.section.is_empty()
    }

    /// Gathered records in arbitrary link order.
    #[inline]
    pub fn entries(&self) -> &[MapRecord<K, V>] {
        self.section.as_slice()
    }
}

impl<K: ConstHash + PartialEq + 'static, V: 'static, P: ProbeStrategy + 'static>
    ScatteredMap<K, V, P>
{
    /// Lookup a value by key.
    #[inline]
    pub fn find(&self, key: &K) -> Option<&V> {
        self.state.table().find(key)
    }

    /// Alias for [`Self::find`].
    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        self.find(key)
    }

    /// True when a key is present in the map.
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        self.find(key).is_some()
    }
}

/// A swiss-table-style lookup table initialized with link-time data. Each item
/// in the map must have a unique hash.
///
/// ## Performance notes
///
/// The map is only ever written to once, and then becomes read-only. This means
/// that we can avoid tombstone logic.
///
/// Metadata is arranged in 16-slot SIMD groups; see [`initialize_scattered_map`].
pub struct ScatteredMapTable<K, V, P: ProbeStrategy = LinearProbe> {
    metadata: *const MetadataSlice,
    num_groups: usize,
    entries: *const MapRecord<K, V>,
    n_entries: usize,
    _marker: PhantomData<(K, V, P)>,
}

impl<K, V, P: ProbeStrategy> ScatteredMapTable<K, V, P> {
    /// # Safety
    /// `metadata` must point to `num_groups` contiguous [`MetadataSlice`] written by
    /// [`initialize_scattered_map`] using the same `entries` slice and probe strategy `P`.
    pub const unsafe fn from_raw(
        metadata: *const MetadataSlice,
        num_groups: usize,
        entries: *const MapRecord<K, V>,
        n_entries: usize,
    ) -> Self {
        Self {
            metadata,
            num_groups,
            entries,
            n_entries,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.n_entries
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.n_entries == 0
    }

    #[inline]
    pub fn num_groups(&self) -> usize {
        self.num_groups
    }
}

impl<K: ConstHash + PartialEq, V, P: ProbeStrategy> ScatteredMapTable<K, V, P> {
    /// SIMD fingerprint match on each group's control bytes, then full hash + key equality.
    ///
    /// Walks the same group sequence as insertion ([`LinearProbe`] by default). There is no early
    /// stop on “some lane empty”: collisions can leave keys in later groups even when earlier
    /// groups have holes.
    pub fn find(&self, key: &K) -> Option<&V> {
        if self.n_entries == 0 || self.num_groups == 0 {
            return None;
        }

        let h = key.hash();
        let tag = control_byte_from_hash(h);
        let mut probe = P::new(self.num_groups, h);

        while let Some(g) = probe.next() {
            unsafe {
                let group = &*self.metadata.add(g);
                let match_mask = group.buckets.simd_eq(u8x16::splat(tag)).to_bitmask();
                let mut bits = match_mask;
                while bits != 0 {
                    let lane = bits.trailing_zeros() as usize;
                    if group.hashes[lane] == h {
                        let row = group.indexes[lane] as usize;
                        let entry = &*self.entries.add(row);
                        if entry.key == *key {
                            return Some(&entry.value);
                        }
                    }
                    bits &= bits - 1;
                }
            }
        }

        None
    }
}

/// Control byte for an occupied slot: low 7 bits = `(hash & 0x7F)`, high bit
/// always set.
///
/// The high bit can be checked via SIMD bitmask.
#[inline]
pub fn control_byte_from_hash(hash: u64) -> u8 {
    ((hash & 0x7F) as u8) | 0x80
}

/// First lane index in `buckets` whose byte is `0` (empty), using SIMD.
#[inline]
fn first_empty_lane(buckets: &u8x16) -> Option<usize> {
    let mask = buckets.simd_eq(u8x16::splat(0)).to_bitmask();
    if mask == 0 {
        None
    } else {
        Some(mask.trailing_zeros() as usize)
    }
}

#[inline]
fn write_slot(group: &mut MetadataSlice, lane: usize, ctrl_byte: u8, hash: u64, row: usize) {
    let mut ctrl = group.buckets.to_array();
    ctrl[lane] = ctrl_byte;
    group.buckets = wide::u8x16::from(ctrl);
    group.hashes[lane] = hash;
    group.indexes[lane] = row as u32;
}

/// Fill `refs` with group metadata for `records`. Row `i` is stored in some `(group, lane)` with
/// control byte [`control_byte_from_hash`] (`0` = empty slot).
///
/// `refs` must be at least [`scattered_map_refs_min_bytes`] for the chosen group count. Leading
/// bytes may be unused alignment padding so that `[MetadataSlice]` is correctly aligned inside the
/// slice.
///
/// Probing matches planned SIMD lookup: group index starts at `(hash >> 7) % num_groups`, then
/// linear scan of groups.
///
/// `refs` must be zeroed before calling this function.
///
/// Returned [`ScatteredMapTable`] aliases `records` and `refs`; keep both alive and unchanged after
/// init.
pub fn initialize_scattered_map<K, V, P: ProbeStrategy>(
    records: &[MapRecord<K, V>],
    refs: &mut [u8],
) -> ScatteredMapTable<K, V, P> {
    let n = records.len();
    if n == 0 {
        return unsafe { ScatteredMapTable::from_raw(core::ptr::null(), 0, records.as_ptr(), 0) };
    }

    let num_groups = num_groups_for_records(n);
    let min_bytes = scattered_map_refs_min_bytes(num_groups);
    assert!(
        refs.len() >= min_bytes,
        "refs too small: need at least {} bytes for {} groups (got {})",
        min_bytes,
        num_groups,
        refs.len()
    );

    let align = align_of::<MetadataSlice>();
    let base = refs.as_mut_ptr() as usize;
    let offset = (align - (base % align)) % align;
    let meta_ptr = unsafe { refs.as_mut_ptr().add(offset) as *mut MetadataSlice };
    let metadata = unsafe { core::slice::from_raw_parts_mut(meta_ptr, num_groups) };

    for (row, record) in records.iter().enumerate() {
        let hash = record.hash;
        let ctrl_byte = control_byte_from_hash(hash);

        let mut probe = P::new(num_groups, hash);

        loop {
            let Some(g) = probe.next() else {
                panic!("no empty slot found (table full)");
            };

            let group = &mut metadata[g];

            if let Some(lane) = first_empty_lane(&group.buckets) {
                write_slot(group, lane, ctrl_byte, hash, row);
                break;
            }
        }
    }

    // Optional later: mirror the first 15 **control** bytes after a flat ctrl region for wrapped
    // SIMD loads. With per-group [`MetadataSlice`], each SIMD load is `group.buckets` only — no
    // cross-group mirror needed for insertion correctness.

    unsafe {
        ScatteredMapTable::from_raw(
            meta_ptr as *const MetadataSlice,
            num_groups,
            records.as_ptr(),
            n,
        )
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __map {
    (@gather $(#[$meta:meta])* $vis:vis static $name:ident: $map:ident < $key:ty, $value:ty >;) => {
        $crate::__map!(@declare_scatter_macro $name, $key, $value, $vis);

        $(#[$meta])*
        $vis static $name: $map<$key, $value> = {
            $crate::__support::link_section::declarative::section!(
                #[section(typed, no_macro)]
                static $name: $crate::__support::link_section::TypedSection<
                    $crate::map::MapRecord<$key, $value>
                >;
            );

            const __MAP_META_BUF_LEN: usize = $crate::map::scattered_map_metadata_reserve_bytes();
            static mut __MAP_META_BUF: [u8; __MAP_META_BUF_LEN] = [0; __MAP_META_BUF_LEN];

            static __MAP_STATE: $crate::map::ScatteredMapState<$key, $value> =
                $crate::map::ScatteredMapState::new();

            $crate::__support::ctor::declarative::ctor!(
                #[ctor(unsafe, anonymous, priority = 0)]
                fn __map_init() {
                    let records = $name.as_slice();
                    let min_bytes = $crate::map::scattered_map_refs_min_bytes(
                        $crate::map::num_groups_for_records(records.len()),
                    );

                    let table = unsafe {
                        assert!(
                            min_bytes <= __MAP_META_BUF_LEN,
                            "map metadata buffer too small: need at least {} bytes, have {}",
                            min_bytes,
                            __MAP_META_BUF_LEN,
                        );
                        __MAP_META_BUF[..min_bytes].fill(0);
                        $crate::map::initialize_scattered_map::<_, _, $crate::map::LinearProbe>(
                            records,
                            &mut __MAP_META_BUF[..min_bytes],
                        )
                    };
                    unsafe {
                        __MAP_STATE.initialize(table);
                    }
                }
            );

            unsafe {
                $crate::map::ScatteredMap::new($name.const_deref(), &__MAP_STATE)
            }
        };
    };
    (@declare_scatter_macro $name:ident, $key:ty, $value:ty, $vis:vis) => {
        $crate::__support::ident_concat!((#[doc(hidden)] #[macro_export] macro_rules!) (
            __ $name __map_scatter__
        ) ({
            ($passthru:tt) => {
                $crate::__map!(@scatter [$key] [$value] $passthru);
            };
        }));

        $crate::__support::ident_concat!(
            (#[doc(hidden)] $vis use)
            (__ $name __map_scatter__)
            (as $name;)
        );
    };
    (scatter $collection:ident => [$key:ty] [$value:ty] $vis:vis $name:ident: $ty:ty = ($key_expr:expr, $value_expr:expr)) => {
        $collection ! (( $collection => $vis static $name: $ty = ($key_expr, $value_expr); ));
    };
    (@scatter [$key:ty] [$value:ty] ($collection:ident => $(#[$imeta:meta])* $vis:vis $kind:ident $name:tt: $ty:ty = ($key_expr:expr, $value_expr:expr);)) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection, type = typed)]
            $(#[$imeta])*
            $vis $kind $name: $crate::map::MapRecord<$key, $value> = $crate::map::MapRecord::new(
                $key_expr,
                $value_expr,
                $crate::const_hash!($key_expr)
            );
        );
    };
}

#[cfg(all(test, not(miri)))]
mod link_tests {
    use super::*;
    use crate::ScatteredMap;

    __map!(@gather pub static TEST_MAP: ScatteredMap<&'static str, u32>;);
    __map!(scatter TEST_MAP => [&'static str] [u32] APPLE: (&'static str, u32) = ("apple", 1));
    __map!(scatter TEST_MAP => [&'static str] [u32] BANANA: (&'static str, u32) = ("banana", 2));

    #[test]
    fn scattered_map_gather_scatter_find() {
        assert_eq!(TEST_MAP.len(), 2);
        assert_eq!(TEST_MAP.find(&"apple"), Some(&1));
        assert_eq!(TEST_MAP.find(&"banana"), Some(&2));
        assert_eq!(TEST_MAP.find(&"orange"), None);
        assert!(TEST_MAP.contains_key(&"apple"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_places_every_record() {
        let a = "apple";
        let b = "banana";
        let records = [
            MapRecord::new(a, 1u32, ConstHasher::<&'static str>::const_hash(a)),
            MapRecord::new(b, 2u32, ConstHasher::<&'static str>::const_hash(b)),
        ];

        let num_groups = num_groups_for_records(records.len());
        let mut refs = vec![0u8; scattered_map_refs_min_bytes(num_groups)];
        initialize_scattered_map::<_, _, LinearProbe>(&records, &mut refs);

        let align = align_of::<MetadataSlice>();
        let base = refs.as_ptr() as usize;
        let offset = (align - (base % align)) % align;
        let meta_ptr = unsafe { refs.as_ptr().add(offset) as *const MetadataSlice };
        let metadata = unsafe { core::slice::from_raw_parts(meta_ptr, num_groups) };

        let mut seen = [false; 2];
        for group in metadata {
            let ctrl = group.buckets.to_array();
            for lane in 0..16 {
                if ctrl[lane] == 0 {
                    continue;
                }
                assert_ne!(ctrl[lane] & 0x80, 0, "occupied slot must have tag bit set");
                assert_eq!(ctrl[lane], control_byte_from_hash(group.hashes[lane]));
                let row = group.indexes[lane] as usize;
                assert!(row < 2, "bad row index");
                assert_eq!(group.hashes[lane], records[row].hash);
                seen[row] = true;
            }
        }
        assert!(seen[0] && seen[1]);
    }

    #[test]
    fn find_returns_values() {
        let a = "apple";
        let b = "banana";
        let records = [
            MapRecord::new(a, 1u32, ConstHasher::<&'static str>::const_hash(a)),
            MapRecord::new(b, 2u32, ConstHasher::<&'static str>::const_hash(b)),
        ];
        let num_groups = num_groups_for_records(records.len());
        let mut refs = vec![0u8; scattered_map_refs_min_bytes(num_groups)];
        let map = initialize_scattered_map::<_, _, LinearProbe>(&records, &mut refs);
        assert_eq!(map.find(&"apple"), Some(&1));
        assert_eq!(map.find(&"banana"), Some(&2));
        assert_eq!(map.find(&"orange"), None);
    }

    #[test]
    fn state_exposes_initialized_table() {
        let a = "apple";
        let records = [MapRecord::new(
            a,
            1u32,
            ConstHasher::<&'static str>::const_hash(a),
        )];
        let num_groups = num_groups_for_records(records.len());
        let mut refs = vec![0u8; scattered_map_refs_min_bytes(num_groups)];
        let table = initialize_scattered_map::<_, _, LinearProbe>(&records, &mut refs);
        let state = ScatteredMapState::new();

        unsafe {
            state.initialize(table);
        }

        assert_eq!(state.table().find(&"apple"), Some(&1));
    }

    #[test]
    fn linear_probe_first_returns_start_group() {
        let len = 8usize;
        let hash = 0xdeadbeef_u64;
        let mut p = LinearProbe::new(len, hash);
        assert_eq!(p.next(), Some(((hash >> 7) as usize) % len));
    }
}
