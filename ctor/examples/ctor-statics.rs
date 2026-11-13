//! Demonstrate various forms of `#[ctor]` statics.
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, Ordering};

use ctor::ctor;

#[derive(Debug)]
#[allow(dead_code)]
struct MyStatic(String);

impl MyStatic {
    fn new(s: impl AsRef<str>) -> Self {
        Self(s.as_ref().to_string())
    }
}

static STATIC_CTOR_REF_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[ctor(unsafe)]
static STATIC_CTOR: MyStatic = MyStatic::new("foo");

#[ctor(unsafe)]
static STATIC_CTOR_REF: &'static MyStatic = &{
    STATIC_CTOR_REF_INITIALIZED.store(true, Ordering::SeqCst);
    MyStatic::new("foo")
};

/// This is not a recommended pattern - just demonstrating how to return static
/// refs.
#[ctor(unsafe)]
#[allow(static_mut_refs)]
static STATIC_CTOR_REF_GLOBAL: &'static MyStatic = {
    unsafe {
        static mut GLOBAL: MaybeUninit<MyStatic> = MaybeUninit::uninit();

        GLOBAL.write(MyStatic::new("foo"));
        GLOBAL.assume_init_ref()
    }
};

/// The above is a similar form to this:
pub static FOO: &'static (dyn ::core::fmt::Debug + Sync) = &"foo";

fn main() {
    assert!(
        STATIC_CTOR_REF_INITIALIZED.load(Ordering::SeqCst),
        "reference static should be initialized before main()"
    );
    println!("STATIC_CTOR: {:?}", STATIC_CTOR);
    println!("STATIC_CTOR_REF: {:?}", STATIC_CTOR_REF);
    println!("STATIC_CTOR_REF_GLOBAL: {:?}", STATIC_CTOR_REF_GLOBAL);
}
