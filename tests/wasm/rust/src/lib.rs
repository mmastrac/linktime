use linktime::{ctor, dtor};

use libc_print::*;

#[ctor(unsafe)]
pub fn ctor() {
    libc_println!("ctor");
}

#[cfg(all(
    // WASI p2 doesn't support custom imports
    not(all(target_os = "wasi", target_env = "p2")), 
    // wasmtime doesn't support custom imports
    not(wasmtime)
))]
mod link_section {
    use linktime::ctor;
    use libc_print::*;
    use linktime::link_section::{section, in_section, TypedSection};

    #[ctor(unsafe, priority = 1)]
    pub fn ctor_slices() {
        libc_println!("ctor_slices:");
        for (idx, s) in SLICES.iter().enumerate() {
            libc_println!("{idx}: {s}");
        }
    }
        
    #[section(typed)]
    pub static SLICES: TypedSection<&'static str>;

    #[in_section(SLICES)]
    pub const SLICE: &'static str = "Hello, world!";

    #[in_section(SLICES)]
    pub const SLICE2: &'static str = "These slices were loaded from the custom section!";

    mod reference {
        use linktime::link_section::{section, in_section, TypedReferenceSection};
        use libc_print::*;

        pub struct Driver {
            pub name: &'static str,
            pub f: fn(),
        }

        impl Driver {
            const fn new(name: &'static str, f: fn()) -> Self {
                Self { name, f }
            }
        }

        #[section(reference)]
        pub static DRIVERS: TypedReferenceSection<Driver>;

        #[in_section(DRIVERS)]
        pub static DRIVER: Driver = Driver::new("driver", || libc_println!("driver"));
    }

    pub fn test_link_section() {
        libc_println!("test_link_section");
        (reference::DRIVER.f)();
        assert!(reference::DRIVERS.offset_of(&reference::DRIVER) == Some(0));
    }
}

#[dtor(unsafe)]
pub fn dtor() {
    libc_println!("dtor");
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

    libc_println!("start");

    #[cfg(all(
        // WASI p2 doesn't support custom imports
        not(all(target_os = "wasi", target_env = "p2")), 
        // wasmtime doesn't support custom imports
        not(wasmtime)
    ))]
    link_section::test_link_section();

    #[cfg(all(target_os = "wasi", target_env = "p2"))]
    {
        unsafe extern "C" {
            fn __wasm_call_dtors();
        }
        unsafe { __wasm_call_dtors(); }
    }
}
