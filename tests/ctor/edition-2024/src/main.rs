#![cfg_attr(linktime_used_linker, feature(used_with_arg))]
//! Edition 2024 test.

use ctor::ctor;
use libc_print::*;

#[ctor]
#[allow(unsafe_code)]
unsafe fn foo() {
    libc_println!("foo");
}

fn main() {
    libc_println!("main");
}
