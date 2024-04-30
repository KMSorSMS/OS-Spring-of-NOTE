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

/// write syscall
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// taskinfo syscall
const SYSCALL_TASK_INFO: usize = 410;
/// syscall num
const SYSCALL_TYPE_NUM: usize = 5;

mod fs;
mod process;

use crate::config::MAX_APP_NUM;
use crate::sync::UPSafeCell;
use crate::task::TASK_MANAGER;
use crate::timer::get_time_ms;
use lazy_static::*;

lazy_static! {
    /// 全局变量：INIT_TIME_LIST用于统计app的初次调度时间
    pub static ref INIT_TIME_LIST: UPSafeCell<[usize;MAX_APP_NUM]> = unsafe {
        UPSafeCell::new([0usize;MAX_APP_NUM])
    };
    /// 全局变量 TASK_INFO_LIST 用于记录每个任务的内容
    pub static ref TASK_INFO_LIST: UPSafeCell<[TaskInfo;MAX_APP_NUM]> = unsafe {
        UPSafeCell::new([TaskInfo::new();MAX_APP_NUM])
    };
}

use fs::*;
use process::*;
const SYSCALL_TYPE: [usize; SYSCALL_TYPE_NUM] = [
    SYSCALL_WRITE,
    SYSCALL_EXIT,
    SYSCALL_YIELD,
    SYSCALL_GET_TIME,
    SYSCALL_TASK_INFO,
];
/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    let current = TASK_MANAGER.get_current_task();
    let mut task_info_list = TASK_INFO_LIST.exclusive_access();
    let current_task_info = &mut task_info_list[current];
    // 先更新系统调用时间
    let init_time_list = INIT_TIME_LIST.exclusive_access();
    current_task_info.set_time(init_time_list[current], get_time_ms());
    // 更新syscall调用,确保syscall在里面
    if SYSCALL_TYPE.contains(&syscall_id) {
        current_task_info.add_syscall_time(syscall_id);
    }
    // drop一定记得
    drop(task_info_list);
    drop(init_time_list);
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TASK_INFO => sys_task_info(args[0] as *mut TaskInfo),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
