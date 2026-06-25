//! A swiss-table-style set initialized with link-time data.
use std::borrow::Borrow;

use crate::{
    ScatteredMap,
    hash::ConstHash,
    map::{__ScatteredMapState, MapRecord},
};

/// A set of items gathered into a set in arbitrary link order.
///
/// See [`ScatteredMap`] for more information about performance and algorithm
/// details.
///
/// ```rust
#[doc = include_str!("../examples/set.rs")]
/// ```
pub struct ScatteredSet<T: 'static> {
    map: ScatteredMap<T, ()>,
}

impl<T: ConstHash + PartialEq + 'static> ScatteredSet<T> {
    #[doc(hidden)]
    pub const fn __new(state: &'static __ScatteredMapState<T, ()>) -> Self {
        Self {
            map: ScatteredMap::__new(state),
        }
    }

    /// The number of records in the set.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// True if the set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// True if the set contains the given key.
    #[inline]
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: ConstHash + Eq + ?Sized,
    {
        self.map.contains_key(key)
    }

    /// Iterate over the values in the set.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.map.keys()
    }

    /// The offset of the record in the set, if it is from this set.
    #[inline]
    pub fn offset_of(this: &Self, key: &SetRecord<T>) -> Option<usize> {
        ScatteredMap::offset_of(&this.map, key.as_map_record_ref())
    }
}

/// One gathered set entry.
#[repr(transparent)]
pub struct SetRecord<T: 'static> {
    record: MapRecord<T, ()>,
}

impl<K: 'static> IntoIterator for &'static ScatteredSet<K> {
    type Item = &'static K;
    type IntoIter =
        ::std::iter::Map<::std::slice::Iter<'static, SetRecord<K>>, fn(&SetRecord<K>) -> &K>;
    #[allow(unsafe_code)]
    fn into_iter(self) -> Self::IntoIter {
        let records: &[SetRecord<K>] = unsafe { core::mem::transmute(self.map.state.records()) };
        records.iter().map(|record| &record.record.key)
    }
}

impl<T: ConstHash + ::core::fmt::Debug + 'static> ::core::fmt::Debug for SetRecord<T> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        ::core::fmt::Debug::fmt(&self.record.key, f)
    }
}

impl<T: ConstHash + 'static> SetRecord<T> {
    #[doc(hidden)]
    pub const fn new(key: T, hash: u64) -> Self {
        Self {
            record: MapRecord::new(key, (), hash),
        }
    }
}

impl<T: ConstHash + 'static> ::core::ops::Deref for SetRecord<T> {
    type Target = MapRecord<T, ()>;
    fn deref(&self) -> &Self::Target {
        &self.record
    }
}

impl<T: ConstHash + 'static> SetRecord<T> {
    #[allow(unsafe_code)]
    pub(crate) fn as_map_record_ref(&self) -> &MapRecord<T, ()> {
        unsafe { core::mem::transmute(&self.record) }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __set {
    (@gather $unique:ident $(#[$meta:meta])* $vis:vis static $name:ident: ($($set:tt)*) < $key:ty >;) => {
        $(#[$meta])*
        $vis static $name: $($set)* <$key> = {
            $crate::__support::link_section::declarative::section!(
                #[section(unsafe, type = typed, name = $name :: $unique)]
                static $name: $crate::__support::link_section::TypedSection<
                    $crate::set::SetRecord<$key>
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

            static __MAP_STATE: $crate::map::__ScatteredMapState<$key, ()> = $crate::map::__ScatteredMapState::new(unsafe { core::mem::transmute($name.const_deref()) }, MAP_META.const_deref());

            $crate::__support::ctor::declarative::ctor!(
                #[ctor(unsafe, anonymous, priority = 0)]
                fn __map_init() {
                    __MAP_STATE.__initialize();
                }
            );

            $crate::set::ScatteredSet::__new(&__MAP_STATE)
        };
    };
    (@scatter [$collection_name:ident :: $unique:ident] [$key:ty] ([$($meta:tt)*] => $(#[$imeta:meta])* $vis:vis $kind:ident $name:tt: $ty:ty = $key_expr:expr;)) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection_name :: $unique, type = typed)]
            $(#[$imeta])*
            $vis $kind $name: $crate::set::SetRecord<$key> = $crate::set::SetRecord::new(
                $key_expr,
                $crate::const_hash!($key_expr)
            );
        );
        $crate::__support::link_section::declarative::in_section!(
            #[in_section(unsafe, name = $collection_name :: $unique :: MAP_META, type = mutable)]
            const _: $crate::map::MapMetadataChunk = $crate::map::MAP_METADATA_CHUNK_ZERO;
        );
    };
}
