use ctor::ctor;

#[ctor(unsafe,link_section = ".ctors")]
fn foo() {
    println!("foo");
}
