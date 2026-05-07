use ctor::ctor;

#[ctor(unsafe, priority = early)]
fn early() {}

#[ctor(unsafe, priority = 1)]
fn priority1() {}

#[ctor(unsafe, priority = 900)]
fn priority900() {}

#[ctor(unsafe, priority = late)]
fn late() {}

#[ctor(unsafe, priority = default)]
fn priority_default() {}

#[ctor(unsafe)]
fn priority_unspecified() {}

#[ctor(unsafe, naked)]
fn naked() {}
