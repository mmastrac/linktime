use linktime::{ctor, dtor};

use libc_print::std_name::println;

#[ctor(unsafe)]
pub fn ctor() {
    println!("ctor");
}

#[dtor(unsafe)]
pub fn dtor() {
    println!("dtor");
}

#[cfg(target_family = "wasm")]
#[unsafe(no_mangle)]
pub extern "C" fn _call_atexit(f: extern "C" fn()) {
    f();
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    println!("start");
    42
}
