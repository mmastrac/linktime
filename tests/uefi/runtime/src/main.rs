//! Exercises ctor / dtor / link_section together and prints the results to
//! COM1.
//!
//! UEFI never runs `.init_array`/`.fini_array` and has no `atexit`, so the
//! entry point drives the lifecycle explicitly via [`linktime::run_constructors`]
//! and [`linktime::run_destructors`]. Everything else uses the ordinary
//! `linktime` API, unchanged from any other platform.

#![no_std]
#![no_main]

#[macro_use]
mod serial;

use core::ffi::c_void;
use core::panic::PanicInfo;

use linktime::{ctor, dtor};

#[ctor(unsafe)]
fn ctor() {
    println!("ctor");
}

mod link_section {
    use linktime::ctor;
    use linktime::link_section::{in_section, section, TypedSection};

    #[ctor(unsafe, priority = 1)]
    fn ctor_slices() {
        println!("ctor_slices:");
        for (idx, s) in SLICES.iter().enumerate() {
            println!("{idx}: {s}");
        }
    }

    #[section(typed)]
    static SLICES: TypedSection<&'static str>;

    #[in_section(SLICES)]
    const SLICE: &'static str = "Hello, world!";

    #[in_section(SLICES)]
    const SLICE2: &'static str = "These slices were loaded from the custom section!";

    mod reference {
        use linktime::link_section::{in_section, section, TypedReferenceSection};

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
        pub static DRIVER: Driver = Driver::new("driver", || println!("driver"));

        // Interleave const/static items.
        #[in_section(DRIVERS)]
        pub const DRIVER_CONST: Driver = Driver::new("driver_const", || println!("driver_const"));

        #[in_section(DRIVERS)]
        pub const DRIVER_CONST2: Driver =
            Driver::new("driver_const2", || println!("driver_const2"));
    }

    mod movable {
        use linktime::ctor;
        use linktime::link_section::{in_section, section, TypedMovableSection};

        #[section(movable)]
        static MOVABLE_LINK_SECTION: TypedMovableSection<u32>;

        #[in_section(MOVABLE_LINK_SECTION)]
        static MOVABLE_40: u32 = 40;
        #[in_section(MOVABLE_LINK_SECTION)]
        static MOVABLE_20: u32 = 20;
        #[in_section(MOVABLE_LINK_SECTION)]
        static MOVABLE_10: u32 = 10;
        #[in_section(MOVABLE_LINK_SECTION)]
        static MOVABLE_30: u32 = 30;

        #[ctor(unsafe, priority = 1)]
        fn ctor() {
            unsafe {
                MOVABLE_LINK_SECTION.sort_unstable();
            }

            assert_eq!(*MOVABLE_40, 40);
            assert_eq!(*MOVABLE_20, 20);
            assert_eq!(*MOVABLE_10, 10);
            assert_eq!(*MOVABLE_30, 30);

            println!(
                "MOVABLE_LINK_SECTION: {:?}",
                MOVABLE_LINK_SECTION.as_slice()
            );
        }
    }

    mod empty {
        use linktime::link_section::{section, TypedSection};

        #[section(typed)]
        pub static EMPTY_LINK_SECTION: TypedSection<u32>;
    }

    pub fn test_link_section() {
        println!("test_link_section");
        println!("DRIVER: {}", reference::DRIVER.name);
        (reference::DRIVER.f)();
        // Submission order interleaves `static` and `const` registrations, so
        // locate DRIVER by its resolved offset rather than assuming an index.
        let drivers = reference::DRIVERS.as_slice();
        assert_eq!(drivers.len(), 3);
        let idx = reference::DRIVERS
            .offset_of(&reference::DRIVER)
            .expect("DRIVER not in section");
        assert_eq!(drivers[idx].name, "driver");
        for (idx, driver) in drivers.iter().enumerate() {
            println!("DRIVERS[{idx}]: {}", driver.name);
            (driver.f)();
        }
        assert!(empty::EMPTY_LINK_SECTION.is_empty());
    }
}

#[dtor(unsafe)]
fn dtor() {
    println!("dtor");
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("panic: {info}");
    serial::shutdown();
}

#[export_name = "efi_main"]
extern "efiapi" fn efi_main(_image_handle: *mut c_void, _system_table: *mut c_void) -> usize {
    serial::init();
    println!("start");

    // UEFI runs no constructors on its own. The second call is a no-op: a
    // duplicated run would print each ctor twice and fail the output match.
    unsafe { linktime::run_constructors() };
    unsafe { linktime::run_constructors() };

    link_section::test_link_section();

    // ... nor destructors.
    unsafe { linktime::run_destructors() };
    unsafe { linktime::run_destructors() };

    serial::shutdown();
}
