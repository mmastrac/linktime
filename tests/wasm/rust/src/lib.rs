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

    #[ctor(unsafe)]
    pub fn ctor_slices() {
        println!("ctor_slices:");
        for slice in SLICES {
            let string = std::str::from_utf8(&slice.as_slice()).unwrap();
            println!("{string}");
        }
    }
        
    #[section]
    pub static SLICES: TypedSection<[u8; 1024]>;

    #[in_section(SLICES)]
    pub static SLICE: [u8; 1024] = string_to_slice("Hello, world!");

    #[in_section(SLICES)]
    pub static SLICE2: [u8; 1024] = string_to_slice("These slices were loaded from the custom section!");

    const fn string_to_slice(string: &str) -> [u8; 1024] {
        let mut slice = [0; 1024];
        slice.split_at_mut(string.len()).0.copy_from_slice(string.as_bytes());
        slice
    }
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
