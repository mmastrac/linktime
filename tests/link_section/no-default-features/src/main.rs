use link_section::declarative::{in_section, section};
use libc_print::*;

section! {
    #[section]
    pub static SECT: link_section::TypedSection<fn()>;
}

in_section! {
    #[in_section(SECT)]
    pub fn _in_section_no_default_features() {
        libc_println!("link-section-no-default-features:in-section");
    }
}

fn main() {
    for f in SECT.as_slice() {
        f();
    }
    libc_println!("link-section-no-default-features:main");
}
