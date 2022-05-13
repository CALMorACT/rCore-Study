use super::{context::TaskContext, switch::__switch};

#[derive(Clone, Copy, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

// TCB (Task Control Block)
#[derive(Clone, Copy)]
pub struct TaskControlBlock {
    pub status: TaskStatus,
    pub task_cx: TaskContext,
}
use crate::{config::MAX_APP_NUM, sync::UPSafeCell};

pub struct TaskMangerInner {
    pub tasks: [TaskControlBlock; MAX_APP_NUM],
    pub current_task: usize,
}
pub struct TaskManger {
    pub num_app: usize,
    pub inner: UPSafeCell<TaskMangerInner>,
}

impl TaskManger {
    pub fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].status = TaskStatus::Ready;
    }

    pub fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        (current_task + 1..self.num_app)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].status == TaskStatus::Ready)
    }

    pub fn run_next_task(&self) {
        if let Some(next_task) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let last_task = inner.current_task;
            inner.tasks[next_task].status = TaskStatus::Running;
            inner.current_task = next_task;
            let last_task_cx_ptr = &mut inner.tasks[last_task].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &mut inner.tasks[next_task].task_cx as *mut TaskContext;
            drop(inner);
            unsafe {
                __switch(last_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            panic!("no task to run, may be All application suspended/exited");
        }
    }
}
