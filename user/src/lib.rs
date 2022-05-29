#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]

#[macro_use]
pub mod console;
mod lang_items;
pub mod syscall;

use crate::syscall::sys_exit;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    sys_exit(main());
    panic!("unreachable after sys_exit!");
}

// 目的在于保护未找到main函数的情况
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Can not find main!");
}
