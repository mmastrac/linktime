//! Edition 2021 test.

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
