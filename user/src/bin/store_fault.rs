#![no_std]
#![no_main]


#[macro_use]
extern crate user;

use user::syscall::sys_yield;
#[no_mangle]
fn main() {
    println!("Into Test store_fault, we will insert an invalid store operation...");
    println!("Kernel should kill this application!");
    for i in 0..10 {
        println!("Fault! [{}/10]", i + 1);
        if i % 5 == 0 {
            sys_yield();
        }
    }
}
