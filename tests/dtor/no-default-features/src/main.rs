use dtor::declarative::dtor;
use libc_print::*;

dtor! {
    #[dtor]
    unsafe fn _dtor_no_default_features() {
        libc_println!("dtor-no-default-features:dtor");
    }
}

fn main() {
    libc_println!("dtor-no-default-features:main");
}
