use crate::config;
use crate::config::KERNEL_STACK_SIZE;
use crate::config::USER_STACK_SIZE;
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

pub fn get_base_i(app_id: usize) -> usize {
    config::APP_BASE_ADDRESS + app_id * config::APP_SIZE_LIMIT
}

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as *const usize).read_volatile() }
}

//[x]: 对于批处理是有效的 
// pub fn init_app_cx(app_id: usize) -> *mut TrapContext {
//     KERNEL_STACK[app_id].push_context(TrapContext::app_init_context(
//         get_base_i(app_id),
//         USER_STACK[app_id].get_sp(),
//     ))
// }

pub fn load_apps() {
    extern "C" {
        fn _num_app();
    }

    let num_app_ptr = _num_app as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };

    // clear i-cache
    unsafe {
        asm!("fence.i");
    }

    // load apps
    for i in 0..num_app {
        let base_i = get_base_i(i);

        (base_i..base_i + config::APP_SIZE_LIMIT)
            .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
        let src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}
