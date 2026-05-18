//! Hash functions for compile-time hashing.

use std::marker::PhantomData;

/// A trait for types that can be hashed at compile time.
pub trait ConstHash {
    /// The hasher for the type.
    type Hasher;
    const HASHER: Self::Hasher;

    fn hash(&self) -> u64;
}

impl ConstHash for &'static str {
    type Hasher = StrHasher;
    const HASHER: Self::Hasher = StrHasher;

    fn hash(&self) -> u64 {
        (Self::HASHER).const_hash(*self)
    }
}

pub struct StrHasher;

impl StrHasher {
    /// Hash a string at compile time.
    pub const fn const_hash(self, s: &'static str) -> u64 {
        xxhash_rust::const_xxh3::xxh3_64(s.as_bytes())
    }
}

pub struct NumericHasher<T: 'static>(PhantomData<T>);

macro_rules! const_hash_numeric {
    ($($ty:ty)*) => {
        $(
            impl ConstHash for $ty {
                type Hasher = NumericHasher<$ty>;
                const HASHER: Self::Hasher = NumericHasher::<$ty>(PhantomData);
                fn hash(&self) -> u64 {
                    (Self::HASHER).const_hash(*self)
                }
            }

            impl NumericHasher<$ty> {
                pub const fn const_hash(self, val: $ty) -> u64 {
                    if ::core::mem::size_of::<$ty>() <= ::core::mem::size_of::<u64>() {
                        let arr = val.to_le_bytes();
                        let mut arr64 = [0u8; 8];
                        let mut i = 0;
                        while i < ::core::mem::size_of::<$ty>() {
                            arr64[i] = arr[i];
                            i += 1;
                        }
                        u64::from_le_bytes(arr64)
                    } else {
                        xxhash_rust::const_xxh3::xxh3_64(&val.to_le_bytes())
                    }
                }
            }
        )*
    }
}

const_hash_numeric!(u8 u16 u32 u64 u128 usize);
const_hash_numeric!(i8 i16 i32 i64 i128 isize);
const_hash_numeric!(f32 f64);

#[macro_export]
macro_rules! const_hash {
    ($val:expr) => {{
        let val = $val;
        const fn hasher<T: $crate::hash::ConstHash>(_: &T) -> T::Hasher {
            T::HASHER
        }
        let hasher = hasher(&val);
        hasher.const_hash(val)
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_const_hash_str() {
        assert_eq!(const_hash!("hello"), 10760762337991515389);
    }

    #[test]
    fn test_const_hash_numeric() {
        assert_eq!(const_hash!(123_u8), 123);
        assert_eq!(const_hash!(123_usize), 123);
        assert_eq!(const_hash!(123.0f32), 1123418112);
        assert_eq!(const_hash!(123.0f64), 4638355772470722560);
    }
}
