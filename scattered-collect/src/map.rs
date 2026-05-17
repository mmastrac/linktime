use core::marker::PhantomData;
use core::mem::{align_of, size_of};

use wide::u8x16;

/// Each item needs a u64 for a hash and a u32 for the index. We add an extra four
/// bytes for slack.
pub type PerItemMetadata = [u8; 16];
pub const PER_ITEM_METADATA_DEFAULT: PerItemMetadata = [0; 16];

/// One SIMD group: 16 control bytes (`0` = empty; occupied uses **7-bit fingerprint** in the low
/// bits with **bit `0x80` set** so a used slot is never byte `0`) plus parallel hash / row index
/// arrays.
#[repr(C)]
pub struct MetadataSlice {
    pub buckets: wide::u8x16,
    pub hashes: [u64; 16],
    pub indexes: [u32; 16],
}

/// How many SIMD groups (each 16 slots) are needed for `n` inserts at ~70% load.
pub fn num_groups_for_records(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let slots = (n as u128 * 10 / 7) as usize + 1;
    let groups = (slots + 15) / 16;
    groups
}

/// Bytes required for `refs` passed to [`initialize_scattered_map`], including alignment padding
/// before the first [`MetadataSlice`].
pub fn scattered_map_refs_min_bytes(num_groups: usize) -> usize {
    if num_groups == 0 {
        return 0;
    }
    (align_of::<MetadataSlice>() - 1) + num_groups * size_of::<MetadataSlice>()
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

/// A swiss-table-style map initialized with link-time data. Each item in the
/// map must have a unique hash.
///
/// ## Performance notes
///
/// The map is only ever written to once, and then becomes read-only. This means
/// that we can avoid tombstone logic.
///
/// Metadata is arranged in 16-slot SIMD groups; see [`initialize_scattered_map`].
pub struct ScatteredMap<K, V, P: ProbeStrategy = LinearProbe> {
    metadata: *const MetadataSlice,
    num_groups: usize,
    entries: *const (K, V, u64),
    n_entries: usize,
    _marker: PhantomData<(K, V, P)>,
}

impl<K, V, P: ProbeStrategy> ScatteredMap<K, V, P> {
    /// # Safety
    /// `metadata` must point to `num_groups` contiguous [`MetadataSlice`] written by
    /// [`initialize_scattered_map`] using the same `entries` slice and probe strategy `P`.
    pub const unsafe fn from_raw(
        metadata: *const MetadataSlice,
        num_groups: usize,
        entries: *const (K, V, u64),
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

impl<K: ConstHash + PartialEq, V, P: ProbeStrategy> ScatteredMap<K, V, P> {
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
                        if entry.0 == *key {
                            return Some(&entry.1);
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
/// Returned [`ScatteredMap`] aliases `records` and `refs`; keep both alive and unchanged after init.
pub fn initialize_scattered_map<K, V, P: ProbeStrategy>(
    records: &[(K, V, u64)],
    refs: &mut [u8],
) -> ScatteredMap<K, V, P> {
    let n = records.len();
    if n == 0 {
        return unsafe { ScatteredMap::from_raw(core::ptr::null(), 0, records.as_ptr(), 0) };
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

    for (row, &(.., hash)) in records.iter().enumerate() {
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
        ScatteredMap::from_raw(
            meta_ptr as *const MetadataSlice,
            num_groups,
            records.as_ptr(),
            n,
        )
    }
}

#[macro_export]
macro_rules! __map {
    (gather $vis:vis $name:ident: $ty:ty) => {
        #[doc(hidden)]
        $crate::__support::ident_concat!(($vis mod) (__ $name _sorted_referenced_slice) ({
            $crate::__support::link_section::declarative::section!(
                #[section(no_macro)]
                pub static $name: $crate::__support::link_section::TypedSection<$ty>;
            );
            $crate::__support::link_section::declarative::section!(
                #[section(aux = $name, no_macro)]
                pub static REFS: $crate::__support::link_section::TypedSection<
                    $crate::map::PerItemMetadata
                >;
            );
            $crate::__support::ctor::declarative::ctor!(#[ctor(unsafe)] unsafe fn __sorted_referenced_slice_init() {
                let main = unsafe { $name.as_mut_slice() };
                let refs = unsafe {
                    let start = REFS.start_ptr_mut();
                    let end = REFS.end_ptr_mut();
                    ::core::slice::from_raw_parts_mut(start, ptr::byte_distance(start, end))
                };
                unsafe {
                    $crate::map::initialize_scattered_map(main, refs);
                }
            });
        }));

        #[doc(hidden)]
        $crate::__support::ident_concat!((#[macro_export] macro_rules!) (__ $name __sorted_referenced_slice_private_macro__) ({
            ($passthru:tt) => {
                $crate::__map!(@scatter $passthru);
            };
        }));

        $crate::__support::ident_concat!((#[doc(hidden)] $vis use) (__ $name __sorted_referenced_slice_private_macro__) (as $name;));

        $vis static $name: $crate::sorted_referenced_slice::ScatteredSortedReferencedSlice<$ty> = {
            $crate::__support::ident_concat!((use ) (__ $name _sorted_referenced_slice) ( as private;));
            unsafe {
                $crate::sorted_referenced_slice::ScatteredSortedReferencedSlice::new(
                    private::$name.const_deref(),
                )
            }
        };
    };
    (scatter $collection:ident => $vis:vis $name:ident: $ty:ty = $expr:expr) => {
        $collection ! (( $collection => $vis $name: $ty = $expr ));
    };
    (@scatter ($collection:ident => $vis:vis $name:ident: $ty:ty = $expr:expr)) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, type = $crate::sorted_referenced_slice::Ref<$ty>, name = REFS, aux = $collection)]
            pub static $name: $crate::sorted_referenced_slice::Ref<$ty> = {
                $crate::__support::link_section::declarative::in_section!(
                    #[in_section(unsafe, type = $ty, name = $collection)]
                    pub static $name: $ty = $expr;
                );
                $crate::sorted_referenced_slice::Ref::new(core::ptr::from_ref(&$name))
            };
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_places_every_record() {
        let a = "apple";
        let b = "banana";
        let records = [
            (a, 1u32, ConstHasher::<&'static str>::const_hash(a)),
            (b, 2u32, ConstHasher::<&'static str>::const_hash(b)),
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
                assert_eq!(group.hashes[lane], records[row].2);
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
            (a, 1u32, ConstHasher::<&'static str>::const_hash(a)),
            (b, 2u32, ConstHasher::<&'static str>::const_hash(b)),
        ];
        let num_groups = num_groups_for_records(records.len());
        let mut refs = vec![0u8; scattered_map_refs_min_bytes(num_groups)];
        let map = initialize_scattered_map::<_, _, LinearProbe>(&records, &mut refs);
        assert_eq!(map.find(&"apple"), Some(&1));
        assert_eq!(map.find(&"banana"), Some(&2));
        assert_eq!(map.find(&"orange"), None);
    }

    #[test]
    fn linear_probe_first_returns_start_group() {
        let len = 8usize;
        let hash = 0xdeadbeef_u64;
        let mut p = LinearProbe::new(len, hash);
        assert_eq!(p.next(), Some(((hash >> 7) as usize) % len));
    }
}
