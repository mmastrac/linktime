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
            pub const _: () = {
                type __InSecStoredTy = <::link_section::TypedSection<
                    ::ctor::collect::Constructor,
                > as ::link_section::__support::SectionItemType>::Item;
                #[link_section = "__DATA,_CTOR0_ISIZE_FN,regular,no_dead_strip"]
                #[used]
                pub static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = ::ctor::collect::Constructor {
                    priority: 500,
                    ctor: __ctor_private,
                };
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
            pub const _: () = {
                type __InSecStoredTy = <::link_section::TypedSection<
                    ::ctor::collect::Constructor,
                > as ::link_section::__support::SectionItemType>::Item;
                #[link_section = "__DATA,_CTOR0_ISIZE_FN,regular,no_dead_strip"]
                #[used]
                pub static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = ::ctor::collect::Constructor {
                    priority: 500,
                    ctor: __ctor_private,
                };
            };
        };
        unsafe { __ctor_private_inner() }
    }
};
