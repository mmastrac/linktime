// Only references `ctor_ran`, so ld has no reason to pull ctor's archive
// member that owns the `__mod_init_func` registration.
extern "C" {
    fn ctor_ran() -> i32;
}

fn main() {
    let ran = unsafe { ctor_ran() };
    match ran {
        0 => println!("DID NOT RUN"),
        1 => println!("RAN"),
        _ => panic!("ctor_ran returned unexpected value: {}", ran),
    }
}
