//! Tests for ctor in an executable that loads a dylib.
#![allow(unused_imports)]

use ctor::ctor;
use dtor::dtor;
use dlopen::raw::Library;
use libc_print::*;

#[allow(unused)]
struct Foo;

impl Foo {
    #[ctor(unsafe)]
    fn ctor() {
        libc_eprintln!("+ Foo::ctor");
    }

    #[dtor(unsafe)]
    fn dtor() {
        libc_eprintln!("- Foo::dtor");
    }
}

#[ctor]
#[cfg(not(test))]
#[allow(unsafe_code)]
unsafe fn ctor() {
    unsafe {
        sleep(1);
    }
    libc_ewriteln!("+ ctor bin");
}

#[dtor]
#[cfg(not(test))]
#[allow(unsafe_code)]
unsafe fn dtor() {
    unsafe {
        sleep(1);
    }
    libc_ewriteln!("- dtor bin");
}

#[cfg(target_vendor = "apple")]
fn lib_extension() -> &'static str {
    "dylib"
}

#[cfg(windows)]
fn lib_extension() -> &'static str {
    "dll"
}

#[cfg(all(not(windows), not(target_vendor = "apple")))]
fn lib_extension() -> &'static str {
    "so"
}

#[cfg(not(windows))]
fn prefix() -> &'static str {
    "lib"
}

#[cfg(windows)]
fn prefix() -> &'static str {
    ""
}

#[cfg(windows)]
#[allow(unsafe_code)]
unsafe extern "C" {
    unsafe fn Sleep(ms: u32);
}

#[cfg(windows)]
#[allow(unsafe_code)]
unsafe fn sleep(seconds: u32) {
    unsafe {
        Sleep(seconds * 1000);
    }
}

#[cfg(not(windows))]
#[allow(unsafe_code)]
unsafe fn sleep(seconds: u32) {
    unsafe {
        libc::sleep(seconds);
    }
}

/// Entry point for the executable.
pub fn main() {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    libc_ewriteln!("++ main start");
    let lib_path = format!(
        "{target_dir}/debug/examples/{}dylib.{}",
        prefix(),
        lib_extension()
    );
    let lib = Library::open(&lib_path)
    .unwrap_or_else(|e| {
        panic!("error opening library {lib_path}: {e:?}");
    });
    drop(lib);
    libc_ewriteln!("-- main end");
}
