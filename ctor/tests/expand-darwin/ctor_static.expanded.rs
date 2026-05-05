use ctor::ctor;
use std::collections::HashMap;
static STATIC_CTOR: ::ctor::statics::Static<HashMap<u32, &'static str>> = {
    #[allow(unsafe_code)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    fn init() -> HashMap<u32, &'static str> {
        unsafe {
            let m = HashMap::new();
            m
        }
    }
    unsafe { ::ctor::statics::Static::<HashMap<u32, &'static str>>::new(init) }
};
const _: () = {
    #[allow(unsafe_code, unused_unsafe)]
    #[link_section = "__TEXT,__text_startup,regular,pure_instructions"]
    extern "C" fn __ctor_private() {
        { _ = &*STATIC_CTOR }
    }
    pub const _: () = {
        type __InSecStoredTy = ::ctor::collect::Constructor;
        #[link_section = "__DATA,_CTOR0_ISIZE_FN,regular,no_dead_strip"]
        #[used]
        pub static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = ::ctor::collect::Constructor {
            priority: 0,
            ctor: __ctor_private,
        };
    };
};
