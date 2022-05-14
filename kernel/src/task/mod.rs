use crate::{
    config,
    loader::{get_num_app, init_app_cx},
    sync::UPSafeCell,
};

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
        // 我们在 static 初始化task时候就会将所有所需的 内核栈和用户栈 初始化出来
        for i in 0..num_app {
            //FIXME: 这一段初始化的 TrapContext 的代码类型可以再整理一下
            tasks[i].task_cx = context::TaskContext::goto_restore(init_app_cx(i) as usize);
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
    TASK_MANAGER.run_next_task();
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}

pub fn start_run_first_task() {
    TASK_MANAGER.run_first_task();
}
