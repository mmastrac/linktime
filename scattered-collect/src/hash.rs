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
    pub const fn const_hash(s: &'static str) -> u64 {
        xxhash_rust::const_xxh3::xxh3_64(s.as_bytes())
    }
}

#[macro_export]
macro_rules! const_hash {
    ($val:expr) => {
        const { $crate::hash::ConstHasher::const_hash($val) }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_const_hash_str() {
        assert_eq!(const_hash!("hello"), 10760762337991515389);
    }
}
