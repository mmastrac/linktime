use ctor::ctor;
use std::collections::HashMap;

#[ctor]
static STATIC_CTOR: &'static HashMap<u32, &'static str> = unsafe {
    let m = HashMap::new();
    m
};
