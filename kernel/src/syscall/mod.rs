const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;

mod fs;
mod process;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => fs::sys_write(args[0], args[1], args[2]),
        SYSCALL_EXIT => process::sys_exit(args[0] as i32),
        _ => panic!("unknown syscall_id: {}", syscall_id),
    }
}
