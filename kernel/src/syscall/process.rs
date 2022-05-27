use crate::{task::{exit_current_and_run_next, suspended_current_and_run_next, current_taskinfo}, timer::get_time_ms};

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    // 不再是运行下一个应用，而是退出继续运行，与下面的暂停运行（sys_yield）相对比
    exit_current_and_run_next();
    panic!("[kernel] [task_exit]Should not reach here");
}

pub fn get_taskinfo() {
    current_taskinfo();
}

pub fn sys_yield() -> isize {
    suspended_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize{
    get_time_ms() as isize
}