//! Compile-fail tests for the `#[scatter]` / `#[gather]` macros.
#![cfg(not(any(miri, bsan)))]

#[test]
#[cfg(not(linktime_used_linker))]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/errors/*.rs");
}
