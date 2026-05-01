use ctor::ctor;
use std::collections::HashMap;
static STATIC_CTOR: ::ctor::statics::Static<HashMap<u32, &'static str>> = {
    #[link_section = "__TEXT_EXEC,initcode"]
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
    #[link_section = "__TEXT_EXEC,initcode"]
    extern "C" fn __ctor_private() {
        { _ = &*STATIC_CTOR }
    }
    #[link_section = "__DATA,_CTOR0_ISIZE_FN,regular,no_dead_strip"]
    #[used]
    pub static CTOR: ::ctor::collect::Constructor = ::ctor::collect::Constructor {
        priority: 0,
        ctor: __ctor_private,
    };
};
