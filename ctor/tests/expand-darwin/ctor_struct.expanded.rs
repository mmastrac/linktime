struct Foo {}
impl Foo {
    #[ctor(unsafe)]
    fn ctor() {}
}
