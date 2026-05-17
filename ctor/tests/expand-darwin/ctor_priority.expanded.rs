use ctor::ctor;
#[allow(dead_code)]
fn early() {
    #[allow(unsafe_code)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    fn __ctor_private_inner() {}
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
            const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = ::ctor::collect::Constructor {
                priority: 101,
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
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn priority1() {
    #[allow(unsafe_code)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    fn __ctor_private_inner() {}
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
            const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = ::ctor::collect::Constructor {
                priority: 1,
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
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn priority900() {
    #[allow(unsafe_code)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    fn __ctor_private_inner() {}
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
            const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = ::ctor::collect::Constructor {
                priority: 900,
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
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn late() {
    #[allow(unsafe_code)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    fn __ctor_private_inner() {}
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
            const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = ::ctor::collect::Constructor {
                priority: (::ctor::collect::LATE),
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
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn priority_default() {
    #[allow(unsafe_code)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    fn __ctor_private_inner() {}
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
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn priority_unspecified() {
    #[allow(unsafe_code)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    fn __ctor_private_inner() {}
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
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn naked() {
    #[allow(unsafe_code)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    fn __ctor_private_inner() {}
    const _: () = {
        #[allow(unsafe_code)]
        #[link_section = "__DATA,__mod_init_func,mod_init_funcs"]
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
