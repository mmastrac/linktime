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
        #[allow(unsafe_code, unused_unsafe)]
        #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
        extern "C" fn __ctor_private() {
            { { __ctor_private_inner() } }
        }
        pub const _: () = {
            type __InSecStoredTy = ::ctor::collect::Constructor;
            #[link_section = "__DATA,_CTOR0_ISIZE_FN,regular,no_dead_strip"]
            #[used]
            pub static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = ::ctor::collect::Constructor {
                priority: 500,
                ctor: __ctor_private,
            };
        };
    };
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn naked_foo() {
    #[allow(unsafe_code)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    fn __ctor_private_inner() {
        {
            ::std::io::_print(format_args!("foo\n"));
        };
    }
    const _: () = {
        #[allow(unsafe_code)]
        #[link_section = "__DATA,__mod_init_func,mod_init_funcs"]
        #[used(linker)]
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
