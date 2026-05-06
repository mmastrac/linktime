use std::sync::atomic::{AtomicUsize, Ordering};
use libc_print::*;

static STATE: AtomicUsize = AtomicUsize::new(0);

#[ctor::ctor(unsafe)]
pub fn init() {
    STATE.fetch_add(1, Ordering::Relaxed);
}

#[ctor::ctor(unsafe, priority = 1)]
pub fn init_2() {
    STATE.fetch_add(1, Ordering::Relaxed);
}

#[cfg(target_family = "wasm")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    assert_eq!(STATE.load(Ordering::Relaxed), 2);
    libc_println!("WASM ran successfully!");
    42
}
