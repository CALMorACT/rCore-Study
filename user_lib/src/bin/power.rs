#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;


#[no_mangle]
fn main() {
    for i in 0..10 {
        println!("Power! [{}/10]", i + 1);
    }
}
