use core::borrow::BorrowMut;

use alloc::vec::Vec;

use crate::config::PAGE_SIZE;

use super::{
    address::{PhysPageNum, StepByOne, VirtAddr, VirtPageNum},
    frame_allocator::{frame_alloc, FrameTracker},
};

bitflags::bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: (ppn.0 << 10) | flags.bits() as usize,
        }
    }

    pub fn empty(&mut self) {
        self.bits = 0;
    }

    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << (56 - PAGE_SIZE)) - 1)).into()
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.bits as u8)
    }

    pub fn set_pte(&mut self, ppn: PhysPageNum, flags: PTEFlags) {
        self.bits = (ppn.0 << 10) | flags.bits() as usize;
    }

    pub fn clear(&mut self) {
        self.bits = 0;
    }

    pub fn is_valid(&self) -> bool {
        // V 非零有效
        !(self.flags() & PTEFlags::V).is_empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
    // entries: [PageTableEntry; 512],
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn,
            frames: alloc::vec![frame],
        }
    }

    fn crete_pte(&mut self, vpn: VirtPageNum) -> Result<&mut PageTableEntry, &str> {
        let indexes = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result = None;
        for i in 0..3 {
            let pte = ppn.get_pte_entry()[indexes[i]].borrow_mut();
            if i == 2 && pte.is_valid() {
                return Err("entry has created");
            }
            if !pte.is_valid() {
                let frame_tracker = frame_alloc().unwrap();
                // 借用会使得这的pte可能为空（伴随临时变量作用域的结束）
                *pte = PageTableEntry::new(frame_tracker.ppn, PTEFlags::V);
                self.frames.push(frame_tracker);
            }
            ppn = pte.ppn();
            result = Some(pte);
        }
        if let Some(pte) = result {
            Ok(pte)
        } else {
            Err("[kernel] [page_table]get pte error")
        }
    }

    fn find_pte(&self, vpn: VirtPageNum) -> Result<&mut PageTableEntry, &str> {
        // 获取三级页表中的对应页表项
        let indexes = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result = None;
        for i in 0..3 {
            let pte = ppn.get_pte_entry()[indexes[i]].borrow_mut();
            if !pte.is_valid() {
                return Err("entry is invalid, maybe vpn not mapped");
            }
            if i == 2 {
                result = Some(pte);
                break;
            }
            ppn = pte.ppn();
        }
        if let Some(pte) = result {
            Ok(pte)
        } else {
            Err("[kernel] [page_table]get pte error")
        }
    }

    pub fn get_pte(&self, vpn: VirtPageNum) -> PageTableEntry {
        match self.find_pte(vpn) {
            Ok(pte) => *pte,
            Err(_) => panic!("[kernel] get_pte failed"),
        }
    }
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        match self.crete_pte(vpn) {
            Ok(pte) => {
                pte.set_pte(ppn, flags);
            }
            Err(e) => {
                panic!("[kernel] map: {}", e);
            }
        }
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        match self.find_pte(vpn) {
            Ok(pte) => {
                pte.clear();
            }
            Err(e) => {
                panic!("[kernel] unmap: {}", e);
            }
        }
    }

    pub fn token(&self) -> usize {
        // 激活 mmu 为 sv39 mode
        8 << 60 | self.root_ppn.0
    }

    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
}
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);

    let mut start = ptr as usize;
    let end = start + len;

    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);

        let mut vpn = start_va.floor();
        let ppn = page_table.get_pte(vpn).ppn();
        vpn.step();

        let mut end_va = VirtAddr::from(vpn);
        end_va = end_va.min(VirtAddr::from(end));

        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_page_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_page_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}
