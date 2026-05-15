use link_section::declarative::{section, in_section};
use link_section::TypedSection;

section! {
    #[section(typed)]
    static FOO: TypedSection<fn()>;
}

in_section! {
    #[in_section(FOO)]
    fn foo() {
        
    }
}

fn main() {
}
