#![cfg_attr(linktime_used_linker, feature(used_with_arg))]
//! `+crt-static` test.

use ctor::ctor;

#[cfg(target_feature = "crt-static")]
#[ctor]
unsafe fn foo() {
    println!("+crt-static");
}

#[cfg(not(target_feature = "crt-static"))]
#[ctor]
unsafe fn foo() {
    println!("-crt-static");
}

fn main() {
    println!("main");
}
