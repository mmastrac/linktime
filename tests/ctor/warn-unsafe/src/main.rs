use ctor::ctor;

/// This should warn
#[ctor]
fn foo() {
    println!("foo");
}

/// This should not warn
#[ctor]
unsafe fn bar() {
    println!("bar");
}

/// This should also not warn
#[ctor(unsafe)]
fn bar2() {
    println!("bar2");
}

#[ctor]
pub static FOO: u32 = {
    println!("side-effect");
    42
};

#[ctor(unsafe)]
pub static FOO_UNSAFE: u32 = {
    println!("side-effect");
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
