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
