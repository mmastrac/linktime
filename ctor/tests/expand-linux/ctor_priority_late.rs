use ctor::ctor;

#[ctor(unsafe, priority = late)]
fn foo() {
    println!("foo");
}
