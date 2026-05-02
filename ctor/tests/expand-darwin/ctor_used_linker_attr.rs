use ctor::ctor;

#[ctor(unsafe, used(linker))]
fn foo() {
    println!("foo");
}

#[ctor(unsafe, naked, used(linker))]
fn naked_foo() {
    println!("foo");
}
