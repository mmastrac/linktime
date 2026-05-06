#[ctor::ctor(unsafe)]
pub static DEBUGGABLE_FUNCTION: &'static (dyn ::core::fmt::Debug + Sync) =
    &{ ::core::fmt::from_fn(|f| f.write_str("debuggable_function")) };

#[ctor::ctor(unsafe)]
pub static DEBUGGABLE_FUNCTION: &'static dyn ::core::fmt::Debug =
    &{ ::core::fmt::from_fn(|f| f.write_str("debuggable_function")) };
    

fn main() {}
