use alloc::sync::Arc;

use crate::{sync::UPSafeCell, mm::memory_set::MemorySet};

pub(crate) mod address;
mod frame_allocator;
mod heap_allocater;
pub(crate) mod memory_set;
pub(crate) mod page_table;

lazy_static::lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> = Arc::new(unsafe {UPSafeCell::new(MemorySet::new_kernel())});
}

pub fn init(){
    heap_allocater::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}