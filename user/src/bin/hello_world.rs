#![no_std]
#![no_main]

#[macro_use]
extern crate user;

use user::syscall::sys_yield;

#[no_mangle]
fn main() {
    for i in 0..10 {
        println!("Hello, world! [{}/10]", i + 1);
        sys_yield();
    }
    println!("Test Hello world OK!");
}
