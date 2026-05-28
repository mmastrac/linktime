use link_section::declarative::{in_section, section};
use libc_print::*;

section! {
    #[section(unsafe, type = typed, name = SECT)]
    pub static SECT: link_section::TypedSection<fn()>;
}

in_section! {
    #[in_section(unsafe, name = SECT, type = typed)]
    pub fn in_section_no_default_features() {
        libc_println!("link-section-no-default-features:in-section");
    }
}

section! {
    #[section(unsafe, type = typed, name = SECT :: AUX)]
    pub static AUX: link_section::TypedSection<fn()>;
}

in_section! {
    #[in_section(unsafe, name = SECT :: AUX, type = typed)]
    pub fn in_section_no_default_features_aux() {
        libc_println!("link-section-no-default-features:in-section-aux");
    }
}

fn main() {
    for f in SECT.as_slice() {
        f();
    }
    for f in AUX.as_slice() {
        f();
    }
    libc_println!("link-section-no-default-features:main");
}
