use ctor::ctor;

#[allow(something_else)]
/// Doc 1
#[ctor]
#[allow(something)]
/// Doc 2
#[cfg(true)]
unsafe fn foo() {
    println!("foo");
}
