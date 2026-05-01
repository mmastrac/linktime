use ctor::ctor;
const _: () = {
    #[allow(dead_code)]
    unsafe fn foo() {
        #[allow(unsafe_code)]
        #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
        unsafe fn __ctor_private_inner() {
            {
                ::std::io::_print(format_args!("foo\n"));
            };
        }
        const _: () = {
            #[allow(unsafe_code, unused_unsafe)]
            #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
            extern "C" fn __ctor_private() {
                { unsafe { __ctor_private_inner() } }
            }
            #[link_section = "__DATA,_CTOR0_ISIZE_FN,regular,no_dead_strip"]
            #[used]
            pub static CTOR: ::ctor::collect::Constructor = ::ctor::collect::Constructor {
                priority: 0,
                ctor: __ctor_private,
            };
        };
        unsafe { __ctor_private_inner() }
    }
};
const _: () = {
    #[allow(dead_code)]
    unsafe fn foo() {
        #[allow(unsafe_code)]
        #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
        unsafe fn __ctor_private_inner() {
            {
                ::std::io::_print(format_args!("foo\n"));
            };
        }
        const _: () = {
            #[allow(unsafe_code, unused_unsafe)]
            #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
            extern "C" fn __ctor_private() {
                { unsafe { __ctor_private_inner() } }
            }
            #[link_section = "__DATA,_CTOR0_ISIZE_FN,regular,no_dead_strip"]
            #[used]
            pub static CTOR: ::ctor::collect::Constructor = ::ctor::collect::Constructor {
                priority: 0,
                ctor: __ctor_private,
            };
        };
        unsafe { __ctor_private_inner() }
    }
};
