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
                        #[cfg(all(target_family = "wasm", target_os = "unknown"))]
                        {
                            static DISARMED: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);
                            if DISARMED.swap(true, ::core::sync::atomic::Ordering::Relaxed) {
                                return;
                            }
                        }
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
