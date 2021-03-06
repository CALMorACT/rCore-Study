use riscv::register::sstatus::{self, Sstatus};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub kernel_satp: usize,  // 内核页表的起始物理地址
    pub kernel_sp: usize,    // 应用在内核栈中栈顶 在内核地址空间中的虚拟地址
    pub trap_handler: usize, // 内核地址空间中，trap_handler 的虚拟地址
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        unsafe {
            sstatus::set_spp(sstatus::SPP::User);
        }
        let mut cx = Self {
            x: [0; 32],
            sstatus: sstatus::read(),
            sepc: entry,
            kernel_satp,
            kernel_sp,
            trap_handler,
        };
        cx.set_sp(sp);
        cx
    }
}
