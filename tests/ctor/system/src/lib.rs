//! Tests for various configurations of the crate.

#![allow(unused_features)]
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]

#[cfg(test)]
mod test {
    use libc_print::*;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::{AtomicBool, Ordering};

    static INITED: AtomicBool = AtomicBool::new(false);
    static INITED_2: AtomicBool = AtomicBool::new(false);

    /// Doc comment
    #[ctor]
    unsafe fn foo() {
        INITED.store(true, Ordering::SeqCst);
    }

    /// This ensures that we support more than of these
    #[ctor]
    unsafe fn foo_2() {
        INITED_2.store(true, Ordering::SeqCst);
    }

    #[cfg(not(target_vendor = "apple"))]
    #[ctor(priority = 2)]
    unsafe fn foo_priority_two() {
        libc_eprintln!("Initialized static with priority 2");
    }

    #[cfg(not(target_vendor = "apple"))]
    #[ctor(priority = 4)]
    unsafe fn foo_priority_four() {
        libc_eprintln!("Initialized static with priority 4");
    }

    #[cfg(not(target_vendor = "apple"))]
    #[ctor(priority = 1)]
    unsafe fn foo_priority_one() {
        libc_eprintln!("Initialized static with priority 1");
    }

    #[cfg(not(target_vendor = "apple"))]
    #[ctor(priority = 3)]
    unsafe fn foo_priority_three() {
        libc_eprintln!("Initialized static with priority 3");
    }

    #[ctor]
    static INITED_3: u8 = unsafe {
        libc_eprintln!("Initialized static");
        42
    };

    /// Override the default link section for Linux
    #[cfg_attr(target_os = "linux", ctor(link_section = ".ctors"))]
    #[cfg_attr(not(target_os = "linux"), ctor)]
    unsafe fn foo_custom_section() {
        // ...
    }

    #[dtor]
    unsafe fn shutdown() {
        // Using println or eprintln here will panic as Rust has shut down
        libc_eprintln!("We don't test shutdown, but if you see this message it worked!");
    }

    #[test]
    fn test_initialized() {
        // Test to see that the ctor ran
        assert!(INITED.load(Ordering::SeqCst));
        assert!(INITED_2.load(Ordering::SeqCst));
        assert_eq!(*INITED_3, 42);
    }
}
