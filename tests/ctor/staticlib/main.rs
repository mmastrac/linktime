// Only references `ctor_ran`, so ld has no reason to pull ctor's archive
// member that owns the `__mod_init_func` registration.
extern "C" {
    fn ctor_ran() -> i32;
}

fn main() {
    let ran = unsafe { ctor_ran() } != 0;
    println!("{}", if ran { "RAN" } else { "DID NOT RUN" });
}
