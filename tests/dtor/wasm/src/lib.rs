#[dtor::dtor(unsafe)]
pub fn shutdown() {
    println!("wasm:dtor");
}

#[cfg(target_family = "wasm")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    println!("wasm:main");
    42
}
