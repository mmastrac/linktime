//! `+crt-static` test.

use ctor::ctor;
use libc_print::*;

#[cfg(target_feature = "crt-static")]
#[ctor]
unsafe fn foo() {
    libc_println!("+crt-static");
}

#[cfg(not(target_feature = "crt-static"))]
#[ctor]
unsafe fn foo() {
    libc_println!("-crt-static");
}

fn main() {
    libc_println!("main");
}
