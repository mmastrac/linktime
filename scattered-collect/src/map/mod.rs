//! A swiss-table-style lookup table initialized with link-time data.
#![doc = concat!("```rust\n", include_str!("../../examples/map.rs"), "\n```\n")]

use link_section::{TypedMutableSection, TypedSection};
use std::{
    borrow::Borrow,
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicU8, Ordering},
};

use crate::hash::ConstHash;
pub use crate::map::table::ScatteredMapTable;
#[cfg(feature = "__internal")]
pub use build::{initialize_scattered_map, safe_byte_count_for_capacity};

mod build;
mod probe;
mod table;

/// One gathered map entry.
///
/// Scatter sites store the hash next to the key and value so map initialization
/// only needs to place rows into metadata slots.
#[repr(C)]
pub struct MapRecord<K, V> {
    /// The key of the record.
    pub key: K,
    /// The value of the record.
    pub value: V,
    /// The hash of the record.
    pub hash: u64,
}

impl<K, V> MapRecord<K, V> {
    /// Create a new map record with a key, value and known `const` hash.
    pub const fn new(key: K, value: V, hash: u64) -> Self {
        Self { key, value, hash }
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

/// One zeroed aux padding block per scatter site.
#[doc(hidden)]
pub type MapMetadataChunk = [u8; build::PER_ITEM_CAPACITY];

/// Zero-filled padding chunk for the map metadata aux section.
#[doc(hidden)]
pub const MAP_METADATA_CHUNK_ZERO: MapMetadataChunk = [0; _];

/// One zeroed aux padding block per scatter site.
#[doc(hidden)]
pub type MapMetadataChunkBase = [u8; build::BASE_CAPACITY];

/// Zero-filled padding chunk for the map metadata aux section.
#[doc(hidden)]
pub const MAP_METADATA_CHUNK_BASE_ZERO: MapMetadataChunkBase = [0; _];

impl<K: ConstHash + PartialEq + 'static, V: 'static> ScatteredMap<K, V> {
    #[doc(hidden)]
    pub const fn __new(state: &'static __ScatteredMapState<K, V>) -> Self {
        Self { state }
    }

    /// Lookup a value by key (deprecated, use `get` instead).
    #[deprecated(since = "0.21.0", note = "Use `get` instead.")]
    #[inline]
    pub fn find<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: ConstHash + Eq + ?Sized,
    {
        self.get(key)
    }

    /// Lookup a value by key.
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: ConstHash + Eq + ?Sized,
    {
        let this = self.state;
        let table = this.ensure_initialized();
        let hash = ConstHash::hash(key);
        let offset = (table.lookup_fn)(table, hash)?;
        let record = &this.records[offset as usize];
        if record.key.borrow() == key {
            Some(&record.value)
        } else {
            None
        }
    }

    /// Iterate over the entries in the map.
    pub fn entries(&self) -> impl Iterator<Item = (&K, &V)> {
        self.state
            .records
            .iter()
            .map(|record| (&record.key, &record.value))
    }

    /// Iterate over the keys in the map.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.state.records.iter().map(|record| &record.key)
    }

    /// Iterate over the values in the map.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.state.records.iter().map(|record| &record.value)
    }

    /// True when a key is present in the map.
    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: ConstHash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }

    /// The number of records in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.state.records.len()
    }

    /// True if the map has no records.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.state.records.is_empty()
    }

    /// The offset of the record in the map, if it is from this map.
    #[inline]
    pub fn offset_of(this: &Self, record: &MapRecord<K, V>) -> Option<usize> {
        this.state.records.offset_of(record)
    }
}

impl<K: 'static, V: 'static> IntoIterator for &'static ScatteredMap<K, V> {
    type Item = (&'static K, &'static V);
    type IntoIter = ::std::iter::Map<
        ::std::slice::Iter<'static, MapRecord<K, V>>,
        fn(&MapRecord<K, V>) -> (&K, &V),
    >;
    fn into_iter(self) -> Self::IntoIter {
        self.state
            .records
            .iter()
            .map(|record| (&record.key, &record.value))
    }
}

/// A swiss-table-style lookup table initialized with link-time data. Each item
/// in the map must have a unique hash.
///
/// For a flat list in arbitrary link order, use [`crate::ScatteredSlice`]. For a sorted
/// list without per-item handles, use [`crate::ScatteredSortedSlice`]. For `static`
/// handles at scatter sites, use [`crate::ScatteredReferencedSlice`]; when sorted order
/// is required as well, use [`crate::ScatteredSortedReferencedSlice`].
///
/// ## Performance notes
///
/// The map's metadata section is only ever written to once, and then becomes
/// read-only. This means that we can avoid tombstone logic.
///
/// Metadata is arranged in 16-slot SIMD groups for optimal performance.
/// ```rust
#[doc = include_str!("../../examples/map.rs")]
/// ```
pub struct ScatteredMap<K: 'static, V: 'static> {
    pub(crate) state: &'static __ScatteredMapState<K, V>,
}

#[doc(hidden)]
pub struct __ScatteredMapState<K: 'static, V: 'static> {
    state: AtomicU8,
    records: &'static TypedSection<MapRecord<K, V>>,
    refs: &'static TypedMutableSection<u8>,
    table: ::core::mem::MaybeUninit<ScatteredMapTable>,
}

impl<K: 'static, V: 'static> __ScatteredMapState<K, V> {
    #[doc(hidden)]
    pub const fn new(
        records: &'static TypedSection<MapRecord<K, V>>,
        refs: &'static TypedMutableSection<u8>,
    ) -> Self {
        Self {
            state: AtomicU8::new(0),
            records,
            refs,
            table: MaybeUninit::uninit(),
        }
    }

    #[doc(hidden)]
    pub fn __initialize(&self) {
        self.ensure_initialized();
    }

    pub(crate) fn records(&self) -> &[MapRecord<K, V>] {
        self.records.as_slice()
    }

    #[allow(unsafe_code)]
    fn ensure_initialized(&self) -> &ScatteredMapTable {
        match self
            .state
            .compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => {
                let table = build::initialize_scattered_map(self.records, unsafe {
                    self.refs.as_mut_slice()
                });
                unsafe { ptr::write(self.table.as_ptr() as _, table) };
                self.state.store(2, Ordering::Relaxed);
                unsafe { self.table.assume_init_ref() }
            }
            Err(2) => unsafe { self.table.assume_init_ref() },
            Err(_) => panic!("Recursive or overlapping initialization of static variable"),
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __map {
    (@gather $unique:ident $(#[$meta:meta])* $vis:vis static $name:ident: ($($map:tt)*) < $key:ty, $value:ty >;) => {
        $(#[$meta])*
        $vis static $name: $($map)* <$key, $value> = {
            $crate::__support::link_section::declarative::section!(
                #[section(unsafe, type = typed, name = $name :: $unique)]
                static $name: $crate::__support::link_section::TypedSection<
                    $crate::map::MapRecord<$key, $value>
                >;
            );

            $crate::__support::link_section::declarative::section!(
                #[section(unsafe, type = mutable, name = $name :: $unique :: MAP_META)]
                static MAP_META: $crate::__support::link_section::TypedMutableSection<u8>;
            );

            $crate::__support::link_section::declarative::in_section!(
                #[in_section(unsafe, type = mutable, name = $name :: $unique :: MAP_META)]
                const _: $crate::map::MapMetadataChunkBase = $crate::map::MAP_METADATA_CHUNK_BASE_ZERO;
            );

            static __MAP_STATE: $crate::map::__ScatteredMapState<$key, $value> = $crate::map::__ScatteredMapState::new($name.const_deref(), MAP_META.const_deref());

            $crate::__support::ctor::declarative::ctor!(
                #[ctor(unsafe, anonymous, priority = 0)]
                fn __map_init() {
                    __MAP_STATE.__initialize();
                }
            );

            $crate::map::ScatteredMap::__new(&__MAP_STATE)
        };
    };
    (@scatter [$collection_name:ident :: $unique:ident] [$key:ty , $value:ty] ([$($meta:tt)*] => $(#[$imeta:meta])* $vis:vis $kind:ident $name:tt: $ty:ty = ($key_expr:expr, $value_expr:expr);)) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection_name :: $unique, type = typed)]
            $(#[$imeta])*
            $vis $kind $name: $crate::map::MapRecord<$key, $value> = $crate::map::MapRecord::new(
                $key_expr,
                $value_expr,
                $crate::const_hash!($key_expr)
            );
        );
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection_name :: $unique :: MAP_META, type = mutable)]
            const _: $crate::map::MapMetadataChunk = $crate::map::MAP_METADATA_CHUNK_ZERO;
        );
    };
    (@scatter [$collection_name:ident :: $unique:ident] [$key:ty , $value:ty] ([$($meta:tt)*] => $(#[$imeta:meta])* $vis:vis $kind:ident $name:tt: $ty:ty = $expr:expr;)) => {
        ::core::compile_error!(
            "invalid #[scatter] syntax for ScatteredMap: expected \
             `static NAME: (Key, Value) = (key_expr, value_expr);` \
             (for example: `static FOO: (&'static str, u32) = (\"foo\", 1);`)"
        );
    };
}

#[cfg(all(test, not(miri)))]
mod link_tests {
    use crate::ScatteredMap;

    __map!(@gather A pub static TEST_MAP: (ScatteredMap)<&'static str, u32>;);
    __map!(@scatter [TEST_MAP::A] [&'static str, u32] ([TEST_MAP] => pub static APPLE: (&'static str, u32) = ("apple", 1);));
    __map!(@scatter [TEST_MAP::A] [&'static str, u32] ([TEST_MAP] => pub static BANANA: (&'static str, u32) = ("banana", 2);));

    #[test]
    fn scattered_map_gather_scatter_get() {
        assert_eq!(TEST_MAP.len(), 2);
        assert_eq!(TEST_MAP.get(&"apple"), Some(&1));
        assert_eq!(TEST_MAP.get(&"banana"), Some(&2));
        assert_eq!(TEST_MAP.get(&"orange"), None);
        assert!(TEST_MAP.contains_key(&"apple"));

        // This works for a borrow from a string too
        let apple = String::from("apple");
        assert_eq!(TEST_MAP.get(apple.as_str()), Some(&1));
    }

    struct Record {
        key: &'static str,
        f: fn(&'static str),
    }

    impl Record {
        pub const fn new(key: &'static str, f: fn(&'static str)) -> Self {
            Self { key, f }
        }

        pub fn call(&self) {
            (self.f)(self.key);
        }
    }

    __map!(@gather A static TEST_MAP_2: (ScatteredMap)<&'static str, Record>;);

    macro_rules! make_test {
        ($($name:ident)*) => {
            $(
            __map!(@scatter [TEST_MAP_2::A] [&'static str, Record] ([TEST_MAP_2] => pub static
                $name: (&'static str, Record) = (
                    stringify!($name),
                    Record::new(stringify!($name), |_key| println!(stringify!($name)))
                );
            ));
            )*
        }
    }

    make_test!(
        A B C D E F G H I J K L M N O P Q R S T U V W X Y Z
        A1 B1 C1 D1 E1 F1 G1 H1 I1 J1 K1 L1 M1 N1 O1 P1 Q1 R1 S1 T1 U1 V1 W1 X1 Y1 Z1
        A2 B2 C2 D2 E2 F2 G2 H2 I2 J2 K2 L2 M2 N2 O2 P2 Q2 R2 S2 T2 U2 V2 W2 X2 Y2 Z2
        A3 B3 C3 D3 E3 F3 G3 H3 I3 J3 K3 L3 M3 N3 O3 P3 Q3 R3 S3 T3 U3 V3 W3 X3 Y3 Z3
        A4 B4 C4 D4 E4 F4 G4 H4 I4 J4 K4 L4 M4 N4 O4 P4 Q4 R4 S4 T4 U4 V4 W4 X4 Y4 Z4
        A5 B5 C5 D5 E5 F5 G5 H5 I5 J5 K5 L5 M5 N5 O5 P5 Q5 R5 S5 T5 U5 V5 W5 X5 Y5 Z5
    );

    #[test]
    fn test_scattered_map_large() {
        TEST_MAP_2.get(&"A").unwrap().call();
        TEST_MAP_2.get(&"B").unwrap().call();
        TEST_MAP_2.get(&"C").unwrap().call();
    }

    __map!(@gather B pub static EMPTY_MAP: (ScatteredMap)<&'static str, u32>;);

    #[test]
    fn test_empty_scattered_map() {
        assert_eq!(EMPTY_MAP.len(), 0);
        assert!(EMPTY_MAP.is_empty());
        assert_eq!(EMPTY_MAP.get(&"apple"), None);
        assert!(!EMPTY_MAP.contains_key(&"apple"));
    }
}
