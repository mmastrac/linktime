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
