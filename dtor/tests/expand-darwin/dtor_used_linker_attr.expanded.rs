use dtor::dtor;
#[allow(dead_code)]
fn foo() {
    fn __dtor_private_inner() {
        {
            ::std::io::_print(format_args!("foo\n"));
        };
    }
    const _: () = {
        #[link_section = "__DATA,__mod_init_func,mod_init_funcs"]
        #[used(linker)]
        static __CTOR_PRIVATE_REF: unsafe extern "C" fn() = {
            unsafe extern "C" fn __ctor_private() {
                ::dtor::__support::at_module_exit(__dtor_private);
            }
            extern "C" fn __dtor_private() {
                { __dtor_private_inner() }
            }
            __ctor_private
        };
    };
    { __dtor_private_inner() }
}
