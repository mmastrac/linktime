use link_section::{section, in_section, TypedSection};

#[section]
static FOO: TypedSection<fn()>;

#[in_section(FOO)]
fn foo() {
    println!("foo");
}

fn main() {
}
