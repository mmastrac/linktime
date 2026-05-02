use dtor::dtor;

#[dtor(unsafe, used(linker))]
fn foo() {
    println!("foo");
}
