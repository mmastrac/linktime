use std::sync::atomic::Ordering;
use crate::RAN;

#[ctor::ctor(unsafe)]
fn install_thing() {
    RAN.store(1, Ordering::SeqCst);
}
