use crate::config::*;
use crate::sync;
use crate::trap::context;
use core::arch::asm;

#[repr(align(4096))]
#[derive(Clone, Copy)]
pub struct KernelStack {
    pub data: [u8; KERNEL_STACK_SIZE],
}

impl KernelStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: context::TrapContext) -> &'static mut context::TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<context::TrapContext>())
            as *mut context::TrapContext;
        unsafe {
            *cx_ptr = cx;
            cx_ptr.as_mut().unwrap()
        }
    }
}
#[repr(align(4096))]
#[derive(Copy, Clone)]
pub struct UserStack {
    pub data: [u8; USER_STACK_SIZE],
}

impl UserStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};

static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

// 批处理系统的实现方式
struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            panic!("[kernel] All application completed!");
        }
        println!("[kernel] load_app: app_{}", app_id);
        // clear icache
        asm!("fence.i");
        // clear app area
        core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        app_dst.copy_from_slice(app_src);
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }
    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

lazy_static::lazy_static! {
    static ref APP_MANAGER: sync::UPSafeCell<AppManager> = unsafe {
        sync:: UPSafeCell::new({
            extern "C" {
                fn _num_app();
            }
            let num_app_ptr = _num_app as *const usize;
            let num_app = num_app_ptr.read_volatile();
            let mut app_start: [usize; MAX_APP_NUM + 1 ] = [0; MAX_APP_NUM + 1];
            let app_start_raw= core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager { num_app, current_app: 0, app_start }
        })
    };
}

pub fn init() {
    APP_MANAGER.exclusive_access().print_app_info();
}

pub fn run_next_app() -> ! {
    let mut app_manger = APP_MANAGER.exclusive_access();
    let current_app = app_manger.get_current_app();
    unsafe {
        app_manger.load_app(current_app);
    }
    app_manger.move_to_next_app();
    drop(app_manger);
    extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe {
        // 通过汇编实现从S到U的特权级跳转
        __restore(
            KERNEL_STACK.push_context(context::TrapContext::app_init_context(
                APP_BASE_ADDRESS,
                USER_STACK.get_sp(),
            )) as *const _ as usize,
        )
    }
    // 理论上正常执行走不到这里，因为我们改动了 sp 指针的位置，这意味着执行代码的顺序和行为都会变化，但是如果我们变化过程失败了，自然就会panic
    panic!("Unreachable in batch::run_current_app!");
}
