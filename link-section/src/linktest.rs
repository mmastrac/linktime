#[link_section = ".text.data.2"]
#[used]
pub static mut DATA1_END: u8 = 1;

#[link_section = ".text.data.1"]
pub static mut DATA1: u8 = 1;
#[link_section = ".text.data.1"]
pub static mut DATA2: u8 = 1;
#[link_section = ".text.data.1"]
pub static mut DATA3: u8 = 1;


pub fn main() {
    unsafe {
        println!("DATA1: {:?}", DATA1);
        println!("DATA2: {:?}", DATA2);
        println!("DATA3: {:?}", DATA3);
        println!("DATA3: {:?}", DATA1_END);
    }
}