use std::sync::atomic::Ordering;
use crate::RAN;

#[ctor::ctor(unsafe)]
fn install_another_thing() {
    RAN.fetch_add(1, Ordering::SeqCst);
}
