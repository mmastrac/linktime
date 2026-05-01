use ctor::ctor;
#[allow(dead_code)]
unsafe fn foo() {
    #[link_section = "__TEXT_EXEC,initcode"]
    unsafe fn __ctor_private_inner() {
        {
            ::std::io::_print(format_args!("foo\n"));
        };
    }
    const _: () = {
        #[allow(unsafe_code)]
        #[link_section = ".ctors"]
        #[used(linker)]
        static __CTOR_PRIVATE_REF: unsafe extern "C" fn() = {
            #[allow(unused_unsafe)]
            #[link_section = "__TEXT_EXEC,initcode"]
            extern "C" fn __ctor_private() {
                { unsafe { __ctor_private_inner() } }
            }
            __ctor_private
        };
    };
    unsafe { __ctor_private_inner() }
}
