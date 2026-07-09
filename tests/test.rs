//! `crok` tests.

#![cfg(not(miri))]

/// `ctor` tests
#[cfg(test)]
mod ctor;

/// `dtor` tests
#[cfg(test)]
mod dtor;

/// `link-section` tests
#[cfg(test)]
mod link_section;

/// WASM runtime tests
#[cfg(test)]
mod wasm;
