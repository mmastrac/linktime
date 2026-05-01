use ctor::ctor;
use std::collections::HashMap;
static STATIC_CTOR: ::ctor::statics::Static<HashMap<u32, &'static str>> = {
    #[link_section = ".text.startup"]
    fn init() -> HashMap<u32, &'static str> {
        unsafe {
            let m = HashMap::new();
            m
        }
    }
    unsafe { ::ctor::statics::Static::<HashMap<u32, &'static str>>::new(init) }
};
const _: () = {
    #[allow(unsafe_code)]
    #[link_section = ".init_array.000"]
    #[used]
    static __CTOR_PRIVATE_REF: unsafe extern "C" fn() = {
        #[allow(unused_unsafe)]
        #[link_section = ".text.startup"]
        extern "C" fn __ctor_private() {
            { _ = &*STATIC_CTOR }
        }
        __ctor_private
    };
};
