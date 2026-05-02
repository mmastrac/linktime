#![cfg_attr(linktime_used_linker, feature(used_with_arg))]
//! Edition 2018 test.

use ctor::ctor;

#[ctor]
#[allow(unsafe_code)]
unsafe fn foo() {
    println!("foo");
}

fn main() {
    println!("main");
}
