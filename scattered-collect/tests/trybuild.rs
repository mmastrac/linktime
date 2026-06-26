//! Compile-fail tests for the `#[scatter]` / `#[gather]` macros.
#![cfg(not(miri))]

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/errors/*.rs");
}
