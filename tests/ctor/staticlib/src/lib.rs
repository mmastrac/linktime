use std::sync::atomic::{AtomicI32, Ordering};

static RAN: AtomicI32 = AtomicI32::new(0);

#[ctor::ctor(unsafe)]
fn install_thing() {
    RAN.store(1, Ordering::SeqCst);
}

#[no_mangle]
pub extern "C" fn ctor_ran() -> i32 {
    RAN.load(Ordering::SeqCst)
}
