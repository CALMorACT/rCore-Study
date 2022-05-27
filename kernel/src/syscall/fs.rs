use crate::task::current_tasktoken;
use crate::mm::page_table::translated_byte_buffer;
const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            // 需要更正获取数据的方式，因为数据在应用的地址空间，内核看不到（buf）的指针地址是应用地址空间的
            let buffers = translated_byte_buffer(current_tasktoken(), buf, len);
            // let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
