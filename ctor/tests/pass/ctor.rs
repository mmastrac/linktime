use ctor::ctor;

/// Doc
#[ctor]
/// Doc
unsafe fn foo() {
}

#[::ctor::ctor]
unsafe fn bar() {
}

#[ctor(unsafe, naked)]
unsafe fn naked() {
}

#[ctor(unsafe, priority = early)]
unsafe fn early() {
}

#[ctor(unsafe, priority = late)]
unsafe fn late() {
}

#[cfg(not(target_vendor = "apple"))]
#[ctor(unsafe, link_section = ".ctors")]
unsafe fn link_section() {
}

#[ctor(unsafe, export_name_prefix = "foo")]
unsafe fn export_name_prefix() {
}

#[ctor]
static STATIC_CTOR: std::collections::HashMap<u32, &'static str> = unsafe {
    let m = std::collections::HashMap::new();
    m
};

fn main() {
}
