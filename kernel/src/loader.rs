use crate::config;
use core::arch::asm;

pub fn get_base_i(app_id: usize) -> usize {
    config::APP_BASE_ADDRESS + app_id * config::APP_SIZE_LIMIT
}

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as *const usize).read_volatile() }
}

pub fn load_app() {
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
