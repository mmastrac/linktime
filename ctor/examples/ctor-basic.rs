//! Matches the `ctor` README introduction (`cargo run --example ctor-basic`).
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]

use ctor::ctor;
use libc_print::*;

#[ctor(unsafe)]
fn foo() {
    libc_println!("Life before main!");
}

fn main() {
    libc_println!("main");
}
