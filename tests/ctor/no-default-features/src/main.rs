use ctor::declarative::ctor;
use libc_print::*;

ctor! {
    #[ctor]
    unsafe fn _ctor_no_default_features() {
        libc_println!("ctor-no-default-features:ctor");
    }
}

fn main() {
    libc_println!("ctor-no-default-features:main");
}
