pub const MAX_APP_NUM: usize = 20;
pub const USER_STACK_SIZE: usize = 4096 * 2; // 8 * . Byte;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000; // 3M Byte

pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;

// Qemu 中设定的时钟频率
pub const CLOCK_FREQ: usize = 12500000;

// 1024 = 2 ^ 10 = 2^2 * 2^4 * 2^4 = 4 * 16^2

pub const PAGE_SIZE: usize = 12;

pub const MEMORY_END: usize = 0x80800000; // 8 MiB App Memory

// 跳板的位置
pub const TRAMPOLINE: usize = usize::MAX - (1 << PAGE_SIZE) + 1;

// trap_context 的情况
pub const TRAP_CONTEXT: usize = TRAMPOLINE - (1 << PAGE_SIZE);

pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + 1 << PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
