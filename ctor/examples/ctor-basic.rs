//! Basic example of using the `ctor` crate.
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]

use ctor::ctor;

#[ctor(unsafe)]
fn ctor() {
    println!("ctor");
}

fn main() {
    println!("main");
}
