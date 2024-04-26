//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    syscall::TASK_INFO_LIST,
    task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, TASK_MANAGER},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

impl TaskInfo {
    pub fn new() -> Self {
        TaskInfo {
            status: TaskStatus::Ready,
            syscall_times: [0; MAX_SYSCALL_NUM],
            time: 0,
        }
    }
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }
    pub fn add_syscall_time(&mut self, syscall: usize) {
        self.syscall_times[syscall] += 1;
    }
    pub fn set_time(&mut self, task_start: usize, task_syscall: usize) {
        self.time = task_syscall - task_start;
    }
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    // 取出当前任务的taskinfo，然后一个个传给指针
    let current = TASK_MANAGER.get_current_task();
    let task_info_list = TASK_INFO_LIST.exclusive_access();
    let current_task_info = &task_info_list[current];
    // 把参数给到指针指向的空间
    unsafe{
        (*_ti).time = current_task_info.time;
        (*_ti).syscall_times = current_task_info.syscall_times;
        (*_ti).status = current_task_info.status;
    }
    drop(task_info_list);
    0
}
