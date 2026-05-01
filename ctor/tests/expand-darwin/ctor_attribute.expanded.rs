use ctor::ctor;
#[allow(dead_code)]
fn foo() {
    #[allow(unsafe_code)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    fn __ctor_private_inner() {
        {
            ::std::io::_print(format_args!("foo\n"));
        };
    }
    const _: () = {
        #[allow(unsafe_code)]
        #[link_section = ".ctors"]
        #[used]
        static __CTOR_PRIVATE_REF: unsafe extern "C" fn() = {
            #[allow(unused_unsafe)]
            #[allow(unsafe_code)]
            #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
            extern "C" fn __ctor_private() {
                { { __ctor_private_inner() } }
            }
            __ctor_private
        };
    };
    { __ctor_private_inner() }
}
