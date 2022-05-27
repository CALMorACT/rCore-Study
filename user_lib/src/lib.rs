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
    clear_bss();
    sys_exit(main());
    panic!("unreachable after sys_exit!");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    // 逐个字节清零
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

// 目的在于保护未找到main函数的情况
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Can not find main!");
}
