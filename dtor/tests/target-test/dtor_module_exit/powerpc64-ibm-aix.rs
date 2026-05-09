use dtor::declarative::dtor;

#[allow(dead_code)]
fn foo_exit() {
    fn __dtor_private_inner() {}
    const _: () =
        {
            #[no_mangle]
            #[export_name =
            "__sinit80000000_expand_probe_expand_probe_$name_L5C1"]
            unsafe extern "C" fn __ctor_private() {
                asm!(".ref __sinit80000000_expand_probe_expand_probe_$name_L5C1",
                    options(preserves_flags, nostack));
                ::dtor::__support::at_module_exit(__dtor_private);
            }
            extern "C" fn __dtor_private() { { __dtor_private_inner() } }
        };
    { __dtor_private_inner() }
}
