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
pub extern "C" fn _start() {
    #[cfg(all(target_os = "wasi", target_env = "p2"))]
    {
        unsafe extern "C" {
            fn __wasm_call_ctors();
        }
        unsafe { __wasm_call_ctors(); }
    }

    println!("start");

    #[cfg(all(target_os = "wasi", target_env = "p2"))]
    {
        unsafe extern "C" {
            fn __wasm_call_dtors();
        }
        unsafe { __wasm_call_dtors(); }
    }
}
