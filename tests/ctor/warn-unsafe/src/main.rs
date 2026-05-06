use ctor::ctor;
use libc_print::*;

/// This should warn
#[ctor]
fn foo() {
    libc_println!("foo");
}

/// This should not warn
#[ctor]
unsafe fn bar() {
    libc_println!("bar");
}

/// This should also not warn
#[ctor(unsafe)]
fn bar2() {
    libc_println!("bar2");
}

#[ctor]
pub static FOO: u32 = {
    libc_println!("side-effect");
    42
};

#[ctor(unsafe)]
pub static FOO_UNSAFE: u32 = {
    libc_println!("side-effect");
    42
};

struct Foo {}

impl Foo {
    #[ctor(unsafe)]
    fn ctor() {}

    #[ctor]
    unsafe fn unsafe_ctor() {}
}

fn main() {}
