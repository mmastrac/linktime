use ctor::declarative::ctor;







#[allow(dead_code)]
fn early() {
    fn __ctor_private_inner() {}
    const _: () =
        {
            #[allow(unused_unsafe, unsafe_code)]
            #[no_mangle]
            #[export_name =
            "__sinit80000101_expand_probe_expand_probe_early_L5C1"]
            extern "C" fn __ctor_private() { { { __ctor_private_inner() } } }
        };
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn priority1() {
    fn __ctor_private_inner() {}
    const _: () =
        {
            #[allow(unused_unsafe, unsafe_code)]
            #[no_mangle]
            #[export_name =
            "__sinit80000001_expand_probe_expand_probe_priority1_L10C1"]
            extern "C" fn __ctor_private() { { { __ctor_private_inner() } } }
        };
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn priority900() {
    fn __ctor_private_inner() {}
    const _: () =
        {
            #[allow(unused_unsafe, unsafe_code)]
            #[no_mangle]
            #[export_name =
            "__sinit80000900_expand_probe_expand_probe_priority900_L15C1"]
            extern "C" fn __ctor_private() { { { __ctor_private_inner() } } }
        };
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn late() {
    fn __ctor_private_inner() {}
    const _: () =
        {
            #[allow(unused_unsafe, unsafe_code)]
            #[no_mangle]
            #[export_name =
            "__sinit89999999_expand_probe_expand_probe_late_L20C1"]
            extern "C" fn __ctor_private() { { { __ctor_private_inner() } } }
        };
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn priority_default() {
    fn __ctor_private_inner() {}
    const _: () =
        {
            #[allow(unused_unsafe, unsafe_code)]
            #[no_mangle]
            #[export_name =
            "__sinit80000500_expand_probe_expand_probe_priority_default_L25C1"]
            extern "C" fn __ctor_private() { { { __ctor_private_inner() } } }
        };
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn priority_unspecified() {
    fn __ctor_private_inner() {}
    const _: () =
        {
            #[allow(unused_unsafe, unsafe_code)]
            #[no_mangle]
            #[export_name =
            "__sinit80000000_expand_probe_expand_probe_priority_unspecified_L30C1"]
            extern "C" fn __ctor_private() { { { __ctor_private_inner() } } }
        };
    { __ctor_private_inner() }
}
#[allow(dead_code)]
fn naked() {
    fn __ctor_private_inner() {}
    const _: () =
        {
            #[allow(unused_unsafe, unsafe_code)]
            #[no_mangle]
            #[export_name =
            "__sinit80000000_expand_probe_expand_probe_naked_L35C1"]
            extern "C" fn __ctor_private() { { { __ctor_private_inner() } } }
        };
    { __ctor_private_inner() }
}
