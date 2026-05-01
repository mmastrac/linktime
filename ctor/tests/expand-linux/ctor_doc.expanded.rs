use ctor::ctor;
/// Doc 1
/// Doc 2
#[allow(dead_code)]
unsafe fn foo() {
    #[link_section = ".text.startup"]
    unsafe fn __ctor_private_inner() {
        {
            ::std::io::_print(format_args!("foo\n"));
        };
    }
    const _: () = {
        #[allow(unsafe_code)]
        #[link_section = ".init_array.000"]
        #[used]
        static __CTOR_PRIVATE_REF: unsafe extern "C" fn() = {
            #[allow(unused_unsafe)]
            #[link_section = ".text.startup"]
            extern "C" fn __ctor_private() {
                { unsafe { __ctor_private_inner() } }
            }
            __ctor_private
        };
    };
    unsafe { __ctor_private_inner() }
}
