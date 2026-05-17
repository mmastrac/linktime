use ctor::ctor;
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
        const _: ::ctor::collect::Constructor = {
            type __InSecStoredTy = <::link_section::TypedSection<
                ::ctor::collect::Constructor,
            > as ::link_section::__support::SectionItemType>::Item;
            const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = ::ctor::collect::Constructor {
                priority: 500,
                ctor: __ctor_private,
            };
            #[used]
            #[link_section = "__DATA,_CTOR0_ISIZE_FN,regular,no_dead_strip"]
            static __LINK_SECTION_CONST_ITEM: ::link_section::__support::SyncUnsafeCell<
                __InSecStoredTy,
            > = ::link_section::__support::SyncUnsafeCell::new(
                __LINK_SECTION_CONST_ITEM_VALUE,
            );
            __LINK_SECTION_CONST_ITEM_VALUE
        };
    };
    unsafe { __ctor_private_inner() }
}
