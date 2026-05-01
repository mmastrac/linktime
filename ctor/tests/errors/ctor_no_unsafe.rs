use ctor::ctor;

#[ctor]
fn foo() {
}

#[ctor(priority = 1)]
fn priority_one() {
}

fn main() {
}
