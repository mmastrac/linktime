use ctor::ctor;

struct MyStatic(String);

impl MyStatic {
    fn new(s: impl AsRef<str>) -> Self {
        Self(s.as_ref().to_string())
    }
}

#[ctor(unsafe)]
static STATIC_CTOR_REF_BAD_1: &'static MyStatic = MyStatic::new("foo");

#[ctor(unsafe)]
static STATIC_CTOR_REF_BAD_2: MyStatic = &MyStatic::new("foo");

fn main() {}
