use ctor::ctor;

#[ctor(priority = bad)]
fn bad_priority() {
}

#[ctor(naked, priority = 1)]
fn cant_have_priority_and_naked() {
}

#[ctor(export_name_prefix = "foo", priority = 1)]
fn cant_have_priority_and_export_name_prefix() {
}

#[ctor(link_section = ".ctors", priority = 1)]
fn cant_have_priority_and_link_section() {
}

fn main() {
}
