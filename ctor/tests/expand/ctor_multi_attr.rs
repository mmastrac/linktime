use ctor::ctor;

#[ctor(used(linker), link_section = ".ctors", body(link_section = ".text.startup"))]
unsafe fn foo() {
    println!("foo");
}
