use ctor::ctor;
#[allow(dead_code)]
fn foo() {
    #[link_section = ".text.startup"]
    fn __ctor_private_inner() {
        {
            ::std::io::_print(format_args!("foo\n"));
        };
    }
    const _: () = {
        #[allow(unsafe_code)]
        #[link_section = ".init_array.65535"]
        #[used]
        static __CTOR_PRIVATE_REF: unsafe extern "C" fn() = {
            #[allow(unused_unsafe)]
            #[link_section = ".text.startup"]
            extern "C" fn __ctor_private() {
                { { __ctor_private_inner() } }
            }
            __ctor_private
        };
    };
    { __ctor_private_inner() }
}
