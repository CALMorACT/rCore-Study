use crate::{config, loader::get_num_app, sync::UPSafeCell};

mod context;
mod switch;
mod task;

lazy_static::lazy_static! {
    /// Global variable: TASK_MANAGER
    pub static ref TASK_MANAGER: task::TaskManger = {
        let num_app = get_num_app();
        let mut tasks = [
            task::TaskControlBlock{
                status: task::TaskStatus::UnInit,
                task_cx: context::TaskContext::zero_init(),
            }; config::MAX_APP_NUM
        ];
        for i in 0..num_app {
            tasks[i].task_cx = context::TaskContext::goto_restore(i);
            tasks[i].status = task::TaskStatus::Ready;
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
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_exited();
}
