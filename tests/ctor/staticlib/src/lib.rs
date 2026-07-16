use std::sync::atomic::{AtomicI32, Ordering};

// Note: the ctor in this file will not be called unless LTO=fat.
mod another_ctor;

pub static RAN: AtomicI32 = AtomicI32::new(0);

#[no_mangle]
pub extern "C" fn ctor_ran() -> i32 {
    RAN.load(Ordering::SeqCst)
}

#[ctor::ctor(unsafe)]
fn install_thing() {
    RAN.fetch_add(1, Ordering::SeqCst);
}
