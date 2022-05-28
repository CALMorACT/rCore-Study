#![no_std]
#![no_main]


#[macro_use]
extern crate user;
use user::syscall::sys_get_time;

#[no_mangle]
fn main() {
    let current_timer = sys_get_time();
    let wait_for = current_timer + 3000;
    while sys_get_time() < wait_for {
        println!("Tick!");
    }
    println!("Test sleep OK!");
}