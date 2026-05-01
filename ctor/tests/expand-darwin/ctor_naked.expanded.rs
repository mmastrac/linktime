use ctor::ctor;
#[allow(dead_code)]
fn naked() {
    #[link_section = "__TEXT_EXEC,initcode"]
    fn __ctor_private_inner() {}
    const _: () = {
        #[allow(unsafe_code)]
        #[link_section = "__DATA,__mod_init_func,mod_init_funcs"]
        #[used]
        static __CTOR_PRIVATE_REF: unsafe extern "C" fn() = {
            #[allow(unused_unsafe)]
            #[link_section = "__TEXT_EXEC,initcode"]
            extern "C" fn __ctor_private() {
                { { __ctor_private_inner() } }
            }
            __ctor_private
        };
    };
    { __ctor_private_inner() }
}
