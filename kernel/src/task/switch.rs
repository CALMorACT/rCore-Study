use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

use super::context::TaskContext;

extern "C" {
    pub fn __switch(
        current_task_context_ptr: *mut TaskContext,
        next_task_context_ptr: *mut TaskContext,
    );
}
