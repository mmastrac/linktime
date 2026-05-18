use hash::ConstHash;
use link_section::{TypedMutableSection, TypedSection};
use std::{
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicU8, Ordering},
};

use crate::map::table::ScatteredMapTable;

mod build;
mod hash;
mod probe;
mod table;

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

/// Chunk size for one aux padding submission (must fit [`scattered_map_metadata_bytes_per_entry`]).
pub const MAP_METADATA_CHUNK_BYTES: usize = 32;

/// One zeroed aux padding block per scatter site.
pub type MapMetadataChunk = [u8; MAP_METADATA_CHUNK_BYTES];

/// Zero-filled padding chunk for the map metadata aux section.
pub const MAP_METADATA_CHUNK_ZERO: MapMetadataChunk = [0; MAP_METADATA_CHUNK_BYTES];

impl<K: ConstHash + PartialEq + 'static, V: 'static> ScatteredMap<K, V> {
    #[doc(hidden)]
    pub const fn __new(state: &'static __ScatteredMapState<K, V>) -> Self {
        Self { state }
    }

    /// Lookup a value by key.
    #[inline]
    pub fn find(&self, key: &K) -> Option<&V> {
        let this = self.state;
        let table = this.ensure_initialized();
        let hash = ConstHash::hash(key);
        let offset = (table.lookup_fn)(&table, hash);
        offset.map(|offset| &this.records[offset as usize].value)
    }

    /// True when a key is present in the map.
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        self.find(key).is_some()
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
pub struct ScatteredMap<K: 'static, V: 'static> {
    state: &'static __ScatteredMapState<K, V>,
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

            $crate::__support::link_section::declarative::section!(
                #[section(mutable, aux(main = $name))]
                $vis static MAP_META: $crate::__support::link_section::TypedMutableSection<u8>;
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
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = MAP_META, aux = $collection, type = mutable)]
            const _: $crate::map::MapMetadataChunk = $crate::map::MAP_METADATA_CHUNK_ZERO;
        );
    };
}

#[cfg(all(test, not(miri)))]
mod link_tests {
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
