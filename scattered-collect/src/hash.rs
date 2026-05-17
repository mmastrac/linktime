const fn const_hash_str(s: &str) -> u64 {
    xxhash_rust::const_xxh3::xxh3_64(s.as_bytes())
}

const fn const_hash_u64(u: u64) -> u64 {
    u
}

const fn const_hash_i32(i: i32) -> u64 {
    i as u64
}

trait ConstHash {
    type Hasher;
}

impl<'a> ConstHash for &'a str {
    type Hasher = StrHasher<'a>;
}
impl ConstHash for u64 {
    type Hasher = U64Hasher;
}
impl ConstHash for i32 {
    type Hasher = I32Hasher;
}

struct ConstHasher<T>(T);

// Specialized impls return type-specific hashers
impl ConstHasher<&str> {
    const fn hasher(&self) -> StrHasher {
        StrHasher(self.0)
    }
}
impl ConstHasher<u64> {
    const fn hasher(&self) -> U64Hasher {
        U64Hasher(self.0)
    }
}
impl ConstHasher<i32> {
    const fn hasher(&self) -> I32Hasher {
        I32Hasher(self.0)
    }
}

// Each hasher has a const fn hash()
struct StrHasher<'a>(&'a str);
impl StrHasher<'_> {
    const fn hash(self) -> u64 {
        const_hash_str(self.0)
    }
}

struct U64Hasher(u64);
impl U64Hasher {
    const fn hash(self) -> u64 {
        const_hash_u64(self.0)
    }
}

struct I32Hasher(i32);
impl I32Hasher {
    const fn hash(self) -> u64 {
        const_hash_i32(self.0)
    }
}

// Fallback via Deref → compile error
impl<T> ::core::ops::Deref for ConstHasher<T> {
    type Target = UnsupportedHasher;
    fn deref(&self) -> &UnsupportedHasher {
        &UnsupportedHasher
    }
}

struct UnsupportedHasher;
impl UnsupportedHasher {
    fn hasher(&self) -> ! {
        panic!("type not supported for const hashing")
    }
}

macro_rules! const_hash {
    ($val:expr) => {
        const { ConstHasher($val).hasher().hash() }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_hash_str() {
        assert_eq!(const_hash!("hello"), 10760762337991515389);
    }
}
