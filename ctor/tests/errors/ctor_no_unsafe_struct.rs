use ctor::ctor;

struct FooGeneric<T> {
    _t: ::std::marker::PhantomData<T>,
}

impl<T: Default> FooGeneric<T> {
    #[ctor]
    fn foo() {
    }
}

fn main() {}
