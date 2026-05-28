use dtor::declarative::dtor;

#[allow(dead_code)]
fn foo() {
    fn __dtor_private_inner() {}
    const _: () =
        {
            #[link_section = ".init_array"]
            #[used]
            static __CTOR_PRIVATE_REF: unsafe extern "C" fn() =
                {
                    unsafe extern "C" fn __ctor_private() {
                        ::dtor::__support::at_binary_exit(__dtor_private);
                    }
                    extern "C" fn __dtor_private() {
                        { __dtor_private_inner() }
                    }
                    __ctor_private
                };
        };
    { __dtor_private_inner() }
}
