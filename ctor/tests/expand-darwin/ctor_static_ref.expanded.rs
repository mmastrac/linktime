use ctor::ctor;
use std::collections::HashMap;
static STATIC_CTOR: ::ctor::statics::Static<&'static HashMap<u32, &'static str>> = {
    #[allow(unsafe_code)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    fn init() -> &'static HashMap<u32, &'static str> {
        return unsafe {
            let m = HashMap::new();
            m
        };
    }
    unsafe { ::ctor::statics::Static::<&'static HashMap<u32, &'static str>>::new(init) }
};
const _: () = {
    #[allow(unsafe_code, unused_unsafe)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    extern "C" fn __ctor_private() {
        { _ = &*STATIC_CTOR }
    }
    /// Force `ld64` to pull the archive member owning `APPLE_PRIORITY_ANCHOR`
    /// (see https://github.com/mmastrac/linktime/issues/496).
    const _: () = {
        mod __ctor_force {}
    };
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
