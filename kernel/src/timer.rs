use riscv::register::time;

use crate::{config::CLOCK_FREQ, sbi::set_timer};

// 用来获得 mtimer 的值
pub fn get_time() -> usize {
    time::read()
}
const TIME_SLICE_COUNT: usize = 100;
pub fn set_next_trigger() {
    // 间隔 10ms 触发一次时钟中断，而由于我们已经将 sie寄存器等正确设置，中断接收，进入 trap_handler 中处理
    // 一旦计数器 mtime 的值超过了 mtimecmp，就会触发中断；我们这个操作是设置 mtimecmp 的值
    set_timer(get_time() + CLOCK_FREQ / TIME_SLICE_COUNT);
}


// 返回微秒时间
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / 1000)
}
