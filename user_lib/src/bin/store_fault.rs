#![no_std]
#![no_main]

use user_lib::syscall::sys_yield;

#[macro_use]
extern crate user_lib;

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
