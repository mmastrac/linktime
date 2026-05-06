//! Demonstrate dynamic `#[ctor]`s.
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]
#![allow(clippy::incompatible_msrv)]

use ctor::ctor;
use libc_print::*;

#[ctor(unsafe)]
static STATIC_CTORS: &[fn()] = const {
    fn bind_const<const N: usize>() {
        libc_eprintln!("bind_const: {}", N);
    }
    [bind_const::<1>, bind_const::<2>, bind_const::<3>].as_slice()
};

#[ctor(unsafe)]
static OPTIONAL_CTOR: &[fn()] = const {
    #[allow(unexpected_cfgs)]
    if cfg!(enable_ctor) {
        fn ctor() {
            libc_eprintln!("ctor");
        }
        [ctor as fn()].as_slice()
    } else {
        &[]
    }
};

fn main() {}
