use ctor::ctor;

#[ctor(unsafe, priority = 1)]
fn foo() {
    println!("foo");
}
