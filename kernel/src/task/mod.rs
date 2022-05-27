use alloc::vec::Vec;

use crate::{
    config,
    loader::{get_num_app, get_app_data},
    sync::UPSafeCell,
    task::task::TaskControlBlock, trap::context::TrapContext,
};

mod context;
mod switch;
mod task;

lazy_static::lazy_static! {
    /// Global variable: TASK_MANAGER
    pub static ref TASK_MANAGER: task::TaskManger = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);

        let mut tasks = Vec::<TaskControlBlock>::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
        }

        task::TaskManger {
            num_app,
            inner: unsafe {
                UPSafeCell::new(task::TaskMangerInner{
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}
pub fn suspended_current_and_run_next() {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}

pub fn start_run_first_task() {
    TASK_MANAGER.run_first_task();
}

pub fn current_taskinfo() -> usize {
    TASK_MANAGER.get_current_task()
}

pub fn current_tasktoken() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}