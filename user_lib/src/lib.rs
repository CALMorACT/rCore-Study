#![no_std]
#![feature(linkage)]

#[no_mangle]
#[link_section = ".text.entry"]

pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    panic!("unreachable after sys_exit!");
}

mod syscall;

// 目的在于保护未找到main函数的情况
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Can not find main!");
}
