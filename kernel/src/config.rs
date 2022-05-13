pub const MAX_APP_NUM: usize = 20;
pub const USER_STACK_SIZE: usize = 4096 * 2; // 8 * 1024 Byte;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
