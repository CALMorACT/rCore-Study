#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
mod console;
mod batch;
mod config;
mod lang_item;
mod loader;
mod sbi;

mod sync;
mod syscall;
mod task;
mod trap;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello, world!");
    trap::init();
    loader::load_apps();
    task::start_run_first_task();
    panic!("Shutdown  machine");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    // 逐个字节清零
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
