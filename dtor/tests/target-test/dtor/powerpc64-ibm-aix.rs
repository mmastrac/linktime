use dtor::declarative::dtor;

#[allow(dead_code)]
fn foo() {
    fn __dtor_private_inner() {}
    const _: () =
        {
            #[no_mangle]
            #[export_name =
            "__sterm80000000_expand_probe_expand_probe_$name_L5C1"]
            extern "C" fn __dtor_private() {
                unsafe {
                    asm!(".ref __sterm80000000_expand_probe_expand_probe_$name_L5C1",
                        options(preserves_flags, nostack));
                }
                { __dtor_private_inner() }
            }
        };
    { __dtor_private_inner() }
}
