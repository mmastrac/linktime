use ctor::ctor;
#[allow(dead_code)]
fn foo() {
    #[link_section = "__TEXT_EXEC,initcode"]
    fn __ctor_private_inner() {
        {
            ::std::io::_print(format_args!("foo\n"));
        };
    }
    const _: () = {
        #[allow(unsafe_code, unused_unsafe)]
        #[link_section = "__TEXT_EXEC,initcode"]
        extern "C" fn __ctor_private() {
            { { __ctor_private_inner() } }
        }
        #[link_section = "__DATA,_CTOR0_ISIZE_FN,regular,no_dead_strip"]
        #[used]
        pub static CTOR: ::ctor::collect::Constructor = ::ctor::collect::Constructor {
            priority: 1,
            ctor: __ctor_private,
        };
    };
    { __ctor_private_inner() }
}
