use core::fmt::{Formatter, self, Debug};

use alloc::vec::Vec;

use crate::{config::MEMORY_END, mm::address::PhysAddr, sync::UPSafeCell};

use super::address::PhysPageNum;

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

// 使用 内核堆 生成了供应用使用的 栈式物理页帧
pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        // 优先尝试从回收队列中取出一个页帧
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            // 回收队列没有页帧，则从当前页帧开始，到结束页帧
            if self.current == self.end {
                // 没有页帧可用，直接返回GG
                None
            } else {
                // 有页帧可用，则返回该页帧(current的值)
                self.current += 1;
                Some((self.current - 1).into())
            }
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        // physical page number
        let ppn = ppn.0;
        // 寻找的 ppn 还未分配过或者已经回收了
        if ppn >= self.current || self.recycled.iter().find(|&v| *v == ppn).is_some() {
            panic!("dealloc invalid frame ppn={:#x}", ppn);
        }
        // 在 recycled 中插入 ppn，表示其被回收
        self.recycled.push(ppn);
    }
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

lazy_static::lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<StackFrameAllocator> =
        unsafe {UPSafeCell::new(StackFrameAllocator::new())};
}
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        Self { ppn }
    }
}
impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        // 在 Tracker 结束的时候就回收其 Tack 的页帧
        frame_dealloc(self.ppn);
    }
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil().into(),
        PhysAddr::from(MEMORY_END).floor().into(),
    )
}

pub fn frame_alloc() -> Option<FrameTracker> {
    // 这里我们希望使用另一个对象包裹 PhysPageNum, 通过这样的方式利用该对象的生命周期（Drop）来回收页帧
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(|ppn| FrameTracker::new(ppn))
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn)
}
