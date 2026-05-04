use linktime::{ctor, dtor};
use linktime::link_section;

#[ctor(unsafe)]

pub extern "C" fn _start() -> i32 {

}
