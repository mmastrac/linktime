use linktime::{ctor, dtor};

use libc_print::std_name::println;

#[ctor(unsafe)]
pub fn ctor() {
    println!("ctor");
}

#[cfg(all(
    // WASI p2 doesn't support custom imports
    not(all(target_os = "wasi", target_env = "p2")), 
    // wasmtime doesn't support custom imports
    not(wasmtime)
))]
mod link_section {
    use linktime::ctor;
    use libc_print::std_name::println;
    use linktime::link_section::{section, in_section, TypedSection};

    #[ctor(unsafe, priority = 1)]
    pub fn ctor_slices() {
        println!("ctor_slices:");
        for (idx, s) in SLICES.iter().enumerate() {
            println!("{idx}: {s}");
        }
    }
        
    #[section]
    pub static SLICES: TypedSection<&'static str>;

    #[in_section(SLICES)]
    pub const SLICE: &'static str = "Hello, world!";

    #[in_section(SLICES)]
    pub const SLICE2: &'static str = "These slices were loaded from the custom section!";
}

#[dtor(unsafe)]
pub fn dtor() {
    println!("dtor");
}

#[cfg(target_family = "wasm")]
#[unsafe(no_mangle)]
pub extern "C" fn _call_atexit(f: extern "C" fn()) {
    f();
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    #[cfg(all(target_os = "wasi", target_env = "p2"))]
    {
        unsafe extern "C" {
            fn __wasm_call_ctors();
        }
        unsafe { __wasm_call_ctors(); }
    }

    println!("start");

    #[cfg(all(target_os = "wasi", target_env = "p2"))]
    {
        unsafe extern "C" {
            fn __wasm_call_dtors();
        }
        unsafe { __wasm_call_dtors(); }
    }
}
