use ctor::ctor;

#[ctor(unsafe, used(linker))]
fn foo() {
    println!("foo");
}
