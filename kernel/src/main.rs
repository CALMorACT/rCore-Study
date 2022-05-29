#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
mod console;
mod config;
mod lang_item;
mod loader;
mod sbi;

mod mm;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

use core::arch::global_asm;

extern crate alloc;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("[Kernel] Hello, world!");
    trap::init();
    loader::load_apps();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
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
