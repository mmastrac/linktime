use libc_print::*;

#[dtor::dtor(unsafe)]
pub fn shutdown() {
    libc_println!("wasm:dtor");
}

#[cfg(target_family = "wasm")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    libc_println!("wasm:main");
    42
}
