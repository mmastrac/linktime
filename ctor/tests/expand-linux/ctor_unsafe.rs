use ctor::ctor;

#[ctor]
unsafe fn foo() {
    println!("foo");
}
