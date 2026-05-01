use ctor::ctor;

#[ctor(unsafe)]
fn foo() -> u32 {
    1
}

#[ctor(unsafe)]
fn foo(_x: u32) {
}

struct Foo;

impl Foo {
    #[ctor(unsafe)]
    fn foo(self) {
    }
}

struct FooGeneric<T> {
    _t: ::std::marker::PhantomData<T>,
}

impl<T: Default> FooGeneric<T> {
    #[ctor(unsafe)]
    fn foo() {
        // can't use generic parameters from outer item
        _ = T::default();
    }
}

fn main() {
}
