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

    mod movable {
        use linktime::ctor;
        use linktime::link_section::{section, in_section, TypedMovableSection};
        use libc_print::*;

        #[section(movable)]
        pub static MOVABLE_LINK_SECTION: TypedMovableSection<u32>;

        #[in_section(MOVABLE_LINK_SECTION)]
        pub static MOVABLE_40: u32 = 40;
        #[in_section(MOVABLE_LINK_SECTION)]
        pub static MOVABLE_20: u32 = 20;
        #[in_section(MOVABLE_LINK_SECTION)]
        pub static MOVABLE_10: u32 = 10;
        #[in_section(MOVABLE_LINK_SECTION)]
        pub static MOVABLE_30: u32 = 30;

        #[ctor(unsafe, priority = 1)]
        pub fn ctor() {
            unsafe {
                MOVABLE_LINK_SECTION.sort_unstable();
            }

            assert_eq!(*MOVABLE_40, 40);
            assert_eq!(*MOVABLE_20, 20);
            assert_eq!(*MOVABLE_10, 10);
            assert_eq!(*MOVABLE_30, 30);

            libc_println!("MOVABLE_LINK_SECTION: {:?}", MOVABLE_LINK_SECTION.as_slice());
        }
    }

    mod empty {
        use linktime::link_section::{section, TypedSection};

        #[section(typed)]
        pub static EMPTY_LINK_SECTION: TypedSection<u32>;
    }

    pub fn test_link_section() {
        libc_println!("test_link_section");
        libc_println!("DRIVER: {}", reference::DRIVER.name);
        (reference::DRIVER.f)();
        assert!(reference::DRIVERS.offset_of(&reference::DRIVER) == Some(0));
        assert!(empty::EMPTY_LINK_SECTION.is_empty());
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
