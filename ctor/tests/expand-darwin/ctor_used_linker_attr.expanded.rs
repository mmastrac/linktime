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
        const _: ::ctor::collect::Constructor = const {
            type __InSecStoredTy = <::link_section::TypedSection<
                ::ctor::collect::Constructor,
            > as ::link_section::__support::SectionItemType>::Item;
            struct __LinkSectionConstItem(::core::cell::UnsafeCell<__InSecStoredTy>);
            unsafe impl Sync for __LinkSectionConstItem {}
            unsafe impl Send for __LinkSectionConstItem {}
            const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = ::ctor::collect::Constructor {
                priority: 500,
                ctor: __ctor_private,
            };
            #[link_section = "__DATA,_CTOR0_ISIZE_FN,regular,no_dead_strip"]
            #[used]
            static __LINK_SECTION_CONST_ITEM: __LinkSectionConstItem = __LinkSectionConstItem(
                ::core::cell::UnsafeCell::new(__LINK_SECTION_CONST_ITEM_VALUE),
            );
            __LINK_SECTION_CONST_ITEM_VALUE
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
