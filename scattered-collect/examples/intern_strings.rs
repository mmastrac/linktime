use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    ops::Deref,
};

use scattered_collect::{
    ScatteredSortedReferencedSlice, gather, scatter, sorted_referenced_slice::Ref,
};

// A string type that allows interning in `O(log N)` time (where `N` is the
// number of interned strings).
enum Str {
    Static(&'static str),
    Interned(usize),
}

impl Str {
    fn try_intern(s: &'static str) -> Option<Self> {
        INTERNED_STRINGS
            .binary_search(&Str::Static(s))
            .ok()
            .map(|i| Str::Interned(i))
    }
}

impl Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.deref(), f)
    }
}

impl Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl Deref for Str {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Str::Static(s) => s,
            Str::Interned(i) => INTERNED_STRINGS[*i].deref(),
        }
    }
}

impl Eq for Str {}
impl PartialEq for Str {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl PartialEq<Ref<Str>> for Str {
    fn eq(&self, other: &Ref<Str>) -> bool {
        self.deref() == &***other
    }
}

impl PartialOrd for Str {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Str {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deref().cmp(other.deref())
    }
}

#[gather]
static INTERNED_STRINGS: ScatteredSortedReferencedSlice<Str>;

macro_rules! intern_string {
    ($name:ident, $string:literal) => {
        #[scatter(INTERNED_STRINGS)]
        static $name: Str = Str::Static($string);
    };
}

intern_string!(HELLO, "hello");
intern_string!(WORLD, "world");

fn main() {
    let hello = Str::try_intern("hello").unwrap();
    let world = Str::try_intern("world").unwrap();
    assert!(Str::try_intern("none").is_none());
    assert_eq!(hello, HELLO);
    assert_eq!(world, WORLD);
    println!("{hello} {world}!");
}
