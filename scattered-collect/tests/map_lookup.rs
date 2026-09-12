//! Lookup coverage for `ScatteredMap`/`ScatteredSet`.

use scattered_collect::{ScatteredSet, gather, map::ScatteredMap, scatter};

#[gather]
static FRUIT: ScatteredMap<&'static str, u32>;

#[scatter(FRUIT)]
static APPLE: (&'static str, u32) = ("apple", 1);

#[scatter(FRUIT)]
static BANANA: (&'static str, u32) = ("banana", 2);

#[scatter(FRUIT)]
static CHERRY: (&'static str, u32) = ("cherry", 3);

#[gather]
static FRUIT_NAMES: ScatteredSet<&'static str>;

#[scatter(FRUIT_NAMES)]
static APPLE_NAME: &'static str = "apple";

#[scatter(FRUIT_NAMES)]
static BANANA_NAME: &'static str = "banana";

#[test]
fn map_get_miss() {
    assert_eq!(FRUIT.get("durian"), None);
    assert_eq!(FRUIT.get(""), None);
    assert_eq!(FRUIT.get("appl"), None);
    assert!(!FRUIT_NAMES.contains("durian"));
}

#[test]
fn map_get_agrees_with_iteration() {
    for (key, value) in &FRUIT {
        assert_eq!(FRUIT.get(*key), Some(value));
    }
}

#[cfg(miri)]
#[test]
fn sections_are_empty_under_miri() {
    assert_eq!(FRUIT.len(), 0);
}

#[cfg(not(miri))]
#[test]
fn map_get_hit() {
    assert_eq!(FRUIT.get("apple"), Some(&1));
    assert_eq!(FRUIT.get("banana"), Some(&2));
    assert_eq!(FRUIT.get("cherry"), Some(&3));
}

#[cfg(not(miri))]
#[test]
fn map_contains_key() {
    assert!(FRUIT.contains_key("apple"));
    assert!(!FRUIT.contains_key("durian"));
}

#[cfg(not(miri))]
#[test]
fn set_contains() {
    assert!(FRUIT_NAMES.contains("apple"));
    assert!(FRUIT_NAMES.contains("banana"));
    assert!(!FRUIT_NAMES.contains("cherry"));
}
