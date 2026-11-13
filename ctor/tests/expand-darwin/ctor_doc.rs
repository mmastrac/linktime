use ctor::ctor;

#[allow(dead_code)]
/// Doc 1
#[ctor]
#[allow(unused_variables)]
/// Doc 2
#[cfg(true)]
unsafe fn foo() {
    println!("foo");
}
