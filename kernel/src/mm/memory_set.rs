use core::{arch::asm, borrow::BorrowMut};

use alloc::{collections::BTreeMap, vec::Vec};
use riscv::register::satp;

use crate::config::{MEMORY_END, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};

use super::{
    address::{PhysAddr, PhysPageNum, SimpleRange, VirtAddr, VirtPageNum},
    frame_allocator::{frame_alloc, FrameTracker},
    page_table::{PTEFlags, PageTable, PageTableEntry},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapType {
    Identifier,
    Framed,
}

bitflags::bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

pub struct MapArea {
    vpn_range: SimpleRange<VirtPageNum>,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        Self {
            vpn_range: SimpleRange::<VirtPageNum>::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            let ppn;
            // 映射到帧
            match self.map_type {
                MapType::Identifier => {
                    ppn = PhysPageNum(vpn.0);
                }
                MapType::Framed => {
                    if let Some(frame) = frame_alloc() {
                        ppn = frame.ppn;
                        self.data_frames.insert(vpn, frame);
                    } else {
                        panic!("[kernel] frame_alloc error when map MapArea");
                    }
                }
            }
            let pte_flags = PTEFlags::from_bits(self.map_perm.bits()).unwrap();
            // 记录到页表
            page_table.map(vpn, ppn, pte_flags);
        }
    }
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            if self.map_type == MapType::Framed {
                self.data_frames.remove(&vpn);
            }
            // TODO: 理论上传入的 page_table 不是 mut 的会报错，但是这里没有
            page_table.unmap(vpn);
        }
    }
    // aligned when load data to memory
    pub fn load_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        // 采用逐字节写入的逻辑，一个内存地址对应一个字节，我们的页表逻辑决定了内存是 4KiB 对齐的(offset 是12位)，一个物理页帧是 4KiB
        for start in (1..data.len()).step_by(1 << PAGE_SIZE) {
            let src = &data[start..data.len().min(start + (1 << PAGE_SIZE))];
            let mut current_vpn = self.vpn_range.get_start();
            let dst =
                page_table.get_pte(current_vpn).ppn().get_page_array()[..src.len()].borrow_mut();
            dst.copy_from_slice(src);
            // 一个页表项映射完成
            current_vpn.0 += current_vpn.0 + 1;
        }
    }
}

// 两者共同构成了一个应用占用的所有物理空间
pub struct MemorySet {
    // 一个可操作的页表
    page_table: PageTable,
    // 应用程序眼中的内存空间
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(self.page_table.borrow_mut());
        if let Some(data) = data {
            map_area.load_data(self.page_table.borrow_mut(), data);
        }
        self.areas.push(map_area);
    }
    pub fn activate(&self) {
        let pt_token = self.page_table.token();
        unsafe {
            satp::write(pt_token);
            asm!("sfence.vma");
        }
    }

    pub fn push_kernel_stack_for_app(
        &mut self,
        start: VirtAddr,
        end: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(MapArea::new(start, end, MapType::Framed, permission), None);
    }

    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        //TODO: map trampoline
        // map kernel sections
        println!(
            "[kernel] .text {:#x}, {:#x}",
            stext as usize, etext as usize
        );
        println!(
            "[kernel] .rodata {:#x}, {:#x}",
            srodata as usize, erodata as usize
        );
        println!(
            "[kernel] .data {:#x}, {:#x}",
            sdata as usize, edata as usize
        );
        println!(
            "[kernel] .bss {:#x}, {:#x}",
            sbss_with_stack as usize, ebss as usize
        );
        println!("---------------");
        println!("mapping .text section");
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identifier,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        println!("mapping .rodata section");
        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identifier,
                MapPermission::R,
            ),
            None,
        );
        println!("mapping .data section");
        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identifier,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping .bss section");
        memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identifier,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        // ekernel map
        println!("mapping ekernel");
        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identifier,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        memory_set
    }

    pub fn load_elf(elf_data: &[u8]) -> Result<(Self, usize, usize), &str> {
        let mut memory_set = Self::new_bare();
        //TODO: map trampoline
        //map elf header
        let elf = xmas_elf::ElfFile::new(elf_data)?;
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let program_header = elf.program_header(i)?;
            if xmas_elf::program::Type::Load == program_header.get_type()? {
                let start_va: VirtAddr = (program_header.virtual_addr() as usize).into();
                let end_va: VirtAddr =
                    ((program_header.virtual_addr() + program_header.mem_size()) as usize).into();
                // get flag
                let mut map_perm = MapPermission::U;
                let ph_flags = program_header.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }

                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(
                    map_area,
                    Some(
                        &elf.input[program_header.offset() as usize
                            ..(program_header.offset() + program_header.file_size()) as usize],
                    ),
                );
            }
        }
        let mut user_stack_bottom: usize = VirtAddr::from(max_end_vpn).into();

        // guard page: 4KiB  leave space for it
        user_stack_bottom += 1 << PAGE_SIZE;

        // map user stack 低266GiB
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );

        // map trap context 在 高256GiB
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAP_CONTEXT.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        Ok((
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        ))
    }

    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::X | PTEFlags::R,
        );
    }
    pub fn get_pte(&self, vpn: VirtPageNum) -> PageTableEntry {
        self.page_table.get_pte(vpn)
    }
}

pub fn test_remap_mem() {
    let mut kernel_space = super::KERNEL_SPACE.exclusive_access();
    let mid_text = VirtAddr::from((stext as usize + etext as usize) / 2);
    let mid_rodata = VirtAddr::from((srodata as usize + erodata as usize) / 2);
    let mid_data = VirtAddr::from((sdata as usize + edata as usize) / 2);
    assert!(!kernel_space.page_table.get_pte(mid_text.floor()).writable());
    assert!(!kernel_space
        .page_table
        .get_pte(mid_rodata.floor())
        .writable());
    assert!(!kernel_space.page_table.get_pte(mid_data.floor()).writable());
    println!("test remap_mem passed!");
}
