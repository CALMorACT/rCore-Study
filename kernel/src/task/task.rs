use alloc::vec::Vec;

use super::{context::TaskContext, switch::__switch};
use crate::config::{kernel_stack_position, TRAP_CONTEXT};
use crate::mm::address::{PhysPageNum, VirtAddr};
use crate::mm::memory_set::{MapPermission, MemorySet};
use crate::mm::KERNEL_SPACE;
use crate::sync::UPSafeCell;
use crate::trap::context::TrapContext;
use crate::trap::trap_handler;

#[derive(Clone, Copy, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

// TCB (Task Control Block)
pub struct TaskControlBlock {
    pub status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize, // 包括应用地址空间中的大小 以及其在堆上分配的大小
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        let mut task_control_block;
        let load_result = MemorySet::load_elf(elf_data);
        match load_result {
            Ok((memory_set, user_sp, entry_point)) => {
                let trap_cx_ppn = memory_set
                    .get_pte(VirtAddr::from(TRAP_CONTEXT).into())
                    .ppn();
                let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
                // push kernel stack
                KERNEL_SPACE.exclusive_access().push_kernel_stack_for_app(
                    kernel_stack_bottom.into(),
                    kernel_stack_top.into(),
                    MapPermission::R | MapPermission::W,
                );
                task_control_block = Self {
                    status: TaskStatus::Ready,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    memory_set,
                    trap_cx_ppn,
                    base_size: user_sp,
                };
                task_control_block.set_trap_cx(TrapContext::app_init_context(
                    entry_point,
                    user_sp,
                    KERNEL_SPACE.exclusive_access().token(),
                    kernel_stack_top,
                    trap_handler as usize,
                ))
            }
            Err(err) => {
                panic!("load elf failed: {}", err);
            }
        }
        task_control_block
    }
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn set_trap_cx(&mut self, cx: TrapContext) {
        *(self.trap_cx_ppn.get_mut::<TrapContext>()) = cx;
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
}

pub struct TaskMangerInner {
    pub tasks: Vec<TaskControlBlock>,
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
            let next_task_cx_ptr = &mut inner.tasks[next_task].task_cx as *const TaskContext;
            drop(inner);
            unsafe {
                __switch(last_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            panic!("no task to run, may be All application suspended/exited");
        }
    }
    pub fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let first_task = &mut inner.tasks[0];
        first_task.status = TaskStatus::Running;
        let next_task_cx_ptr = &first_task.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("[Kernel] [run_first_task]should not reach here");
    }

    pub fn get_current_task(&self) -> usize {
        self.inner.exclusive_access().current_task
    }
    pub fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }
    pub fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }
}
