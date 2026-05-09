use ctor::declarative::ctor;

#[allow(dead_code)]
fn foo() {
    fn __ctor_private_inner() {}
    const _: () =
        {
            #[allow(unused_unsafe, unsafe_code)]
            #[no_mangle]
            #[export_name =
            "__sinit80000000_expand_probe_expand_probe_foo_L5C1"]
            extern "C" fn __ctor_private() {
                unsafe {
                    asm!(".ref __sinit80000000_expand_probe_expand_probe_foo_L5C1",
                        options(preserves_flags, nostack));
                }
                { { __ctor_private_inner() } }
            }
        };
    { __ctor_private_inner() }
}
