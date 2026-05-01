//! Tests for ctor in dylibs.
#![allow(dead_code, unused_imports, unused_features, unsafe_code)]
#![cfg_attr(feature = "used_linker", feature(used_with_arg))]

use ctor::ctor;
use dtor::dtor;
use libc_print::*;

#[cfg(never)]
#[ctor(unsafe)]
fn never() {
    libc_ewriteln!("+++ ctor never run");
}

#[cfg(never)]
#[ctor(unsafe)]
static NEVER_STATIC: u8 = unsafe {
    libc_ewriteln!("+++ ctor static never run");
    42
};

#[cfg(never)]
#[dtor(unsafe)]
fn never() {
    libc_ewriteln!("+++ dtor never run");
}

#[cfg(windows)]
unsafe extern "C" {
    #[allow(unused)]
    unsafe fn Sleep(ms: u32);
}

#[cfg(windows)]
unsafe fn sleep(seconds: u32) {
    unsafe {
        Sleep(seconds * 1000);
    }
}

#[cfg(not(windows))]
unsafe fn sleep(seconds: u32) {
    unsafe {
        libc::sleep(seconds);
    }
}

#[ctor(unsafe)]
static STATIC_INT: u8 = {
    libc_ewriteln!("+++ ctor STATIC_INT");
    200
};

#[ctor(unsafe)]
#[cfg(not(test))]
#[cfg(target_feature = "crt-static")]
unsafe fn ctor() {
    unsafe {
        sleep(1);
    }
    libc_ewriteln!("+++ ctor lib (+crt-static)");
}

#[ctor]
#[cfg(not(test))]
#[cfg(not(target_feature = "crt-static"))]
#[allow(unsafe_code)]
unsafe fn ctor() {
    unsafe {
        sleep(1);
    }
    libc_ewriteln!("+++ ctor lib (-crt-static)");
}

#[dtor]
#[cfg(not(test))]
unsafe fn dtor() {
    unsafe {
        sleep(1);
    }
    libc_ewriteln!("--- dtor lib");
}
