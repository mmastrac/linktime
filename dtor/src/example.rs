//! This example demonstrates the various types of ctor/dtor in an executable
//! context.
#![recursion_limit = "60"]
use dtor::dtor;
use libc_print::*;

#[dtor(unsafe)]
fn dtor() {
    libc_eprintln!("dtor");
}

#[dtor(unsafe, method = at_module_exit)]
#[allow(unsafe_code)]
fn dtor_at_module_exit() {
    libc_eprintln!("at_module_exit");
}

/// This one is unsafe.
#[cfg(not(target_os = "windows"))] // unsupported on Windows
#[dtor(method = at_binary_exit)]
#[allow(unsafe_code)]
unsafe fn dtor_at_binary_exit() {
    libc_eprintln!("at_binary_exit");
}

/// Custom link section (note that Apple's mach-o linker requires a specific link section format).
#[cfg_attr(not(target_vendor = "apple"), dtor(unsafe, method = linker, link_section = ".manual.dtor"))]
#[cfg_attr(target_vendor = "apple", dtor(unsafe, method = linker, link_section = "__DATA,__manual_dtors"))]
fn dtor_link_section() {
    libc_eprintln!("link_section");
}

#[cfg_attr(not(target_vendor = "apple"), dtor(unsafe, method = linker, link_section = ".dtors", crate_path = ::dtor))]
#[cfg_attr(target_vendor = "apple", dtor(unsafe, method = linker, link_section = "__DATA,__manual_dtors", crate_path = ::dtor))]
fn bar() {
    println!("foo");
}

/// Demonstrating some of the various types of [`dtor`].
pub fn main() {
    use libc_print::*;
    libc_eprintln!("main!");
}
