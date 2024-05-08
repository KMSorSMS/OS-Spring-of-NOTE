//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.
/// read syscall
const SYSCALL_READ: usize = 63;
/// write syscall
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// setpriority syscall
const SYSCALL_SET_PRIORITY: usize = 140;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// getpid syscall
const SYSCALL_GETPID: usize = 172;
/// sbrk syscall
const SYSCALL_SBRK: usize = 214;
/// munmap syscall
const SYSCALL_MUNMAP: usize = 215;
/// fork syscall
const SYSCALL_FORK: usize = 220;
/// exec syscall
const SYSCALL_EXEC: usize = 221;
/// mmap syscall
const SYSCALL_MMAP: usize = 222;
/// waitpid syscall
const SYSCALL_WAITPID: usize = 260;
/// spawn syscall
const SYSCALL_SPAWN: usize = 400;
/// taskinfo syscall
const SYSCALL_TASK_INFO: usize = 410;

mod fs;
mod process;
use alloc::collections::BTreeMap;
use lazy_static::*;

//建立两个全局变量，采取BTeeMap的数据结构存储任务的pid和信息的对应关系
lazy_static! {
    ///全局变量：INIT_TIME_LIST用于统计app的初次调度时间
    pub static ref INIT_TIME_LIST: UPSafeCell<BTreeMap<usize,usize>> = unsafe {
        UPSafeCell::new(BTreeMap::new())
    };
    ///全局变量TASK_INFO_LIST 用于记录每个任务的info
    pub static ref TASK_INFO_LIST: UPSafeCell<BTreeMap<usize,TaskInfo>> = unsafe {
        UPSafeCell::new(BTreeMap::new())
    };
}
pub use process::TaskInfo;

use fs::*;
use process::*;

use crate::{config::MAX_SYSCALL_NUM, sync::UPSafeCell, task::current_task, timer::get_time_ms};
/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    //得到当前任务的pid号
    // let current_block = current_task().unwrap();
    let current = current_task().unwrap().pid.0; 
    // drop(current_block);
    let mut task_info_list = TASK_INFO_LIST.exclusive_access();
    let current_task_info = task_info_list.get_mut(&current).unwrap();
    // 先更新系统调用时间
    let init_time_list = INIT_TIME_LIST.exclusive_access();
    current_task_info.set_time(*init_time_list.get(&current).unwrap(), get_time_ms());
    // 更新syscall调用,确保syscall在里面
    if syscall_id < MAX_SYSCALL_NUM {
        // if syscall_id == 169{
        //     println!("\ncurrent task is{} ",current);
        //     println!("\n--{}--\n", current_task_info.syscall_times[169])
        // }
        current_task_info.add_syscall_time(syscall_id);
    }
    // drop一定记得
    drop(task_info_list);
    drop(init_time_list);
    match syscall_id {
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TASK_INFO => sys_task_info(args[0] as *mut TaskInfo),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_SBRK => sys_sbrk(args[0] as i32),
        SYSCALL_SPAWN => sys_spawn(args[0] as *const u8),
        SYSCALL_SET_PRIORITY => sys_set_priority(args[0] as isize),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
