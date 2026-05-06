use dtor::dtor;
use libc_print::*;

/// This will not be called in all cases.
#[dtor(method = linker)]
unsafe fn _dtor_no_default_features() {
    libc_println!("dtor-link-section:dtor");
}

fn main() {
    libc_println!("dtor-link-section:main");
}
