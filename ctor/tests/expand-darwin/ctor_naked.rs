use ctor::ctor;

#[ctor(unsafe, naked)]
fn naked() {}
