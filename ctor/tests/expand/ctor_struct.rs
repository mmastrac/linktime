use libc_print::*;

struct Foo {
}

impl Foo {
    #[ctor(unsafe, link_section = ".ctors", body(link_section = ".text.startup"))]
    fn ctor() {
        libc_eprintln!("Foo::ctor");
    }
}
