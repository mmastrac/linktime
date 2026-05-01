use libc_print::*;
struct Foo {}
impl Foo {
    #[ctor(unsafe, link_section = ".ctors", body(link_section = ".text.startup"))]
    fn ctor() {
        {
            #[allow(unused_must_use)]
            {
                let mut stm = ::libc_print::__LibCWriter::new(
                    ::libc_print::__LIBC_STDERR,
                );
                stm.write_fmt(format_args!("Foo::ctor"));
                stm.write_nl();
            }
        };
    }
}
