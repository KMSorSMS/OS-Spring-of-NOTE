//! Process management syscalls
use core::mem::size_of;

use crate::{
    config::MAX_SYSCALL_NUM,
    mm::translated_byte_buffer,
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus,
    },
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
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    // 思路简单，我们需要把传进来的用户的虚拟地址_ts对应的物理地址塞入正确的数据
    //正确的数据计算方法和之前一样，照抄，所以最关键的是得到虚拟地址对应的物理地址，，参考sys_write实现，特别是translated_byte_buffer
    let mut buffers =
        translated_byte_buffer(current_user_token(), _ts as *const u8, size_of::<TimeVal>());
    //通过复用translated_byte_buffer我们就可以做到如果TimeVal split了，我们是按照一字节一字节塞入数据的
    let us = get_time_us();
    let sec = us / 1_000_000;
    let usec = us % 1_000_000;
    //一字节一字节塞入数据,按照小端存储，先存sec再存usec
    // 将sec的值写入到buffers中
    let sec_bytes = sec.to_le_bytes(); //将sec转为字节切片
    let usec_bytes = usec.to_le_bytes(); //将usec转为字节切片
    if buffers[0].len() < 8 {
        //如果存放在sec这里就跨页了，那么就需要向后走一页
        let len = buffers[0].len();
        buffers[0].copy_from_slice(&sec_bytes[..len]);
        //剩下的部分需要复制到buffer[1]里面
        buffers[1][..(8-len)].copy_from_slice(&sec_bytes[len..]);
        //后续的可以直接放到下一页
        buffers[1][(8-len)..].copy_from_slice(&usec_bytes);
    } else if buffers[0].len() < 16 {
        //走到这里是当前页剩余超过8字节，但是少于16字节，导致usec放的时候会跨页
        buffers[0][..8].copy_from_slice(&sec_bytes);
        // 当前页能放置usec的长度就是buffer[0]的长度减去sec_bytes的长度（8）
        let len = buffers[0].len() - 8;
        buffers[0][8..].copy_from_slice(&usec_bytes[..len]);
        buffers[1][..(8-len)].copy_from_slice(&usec_bytes[len..]);
    }else {
        //这种情况就是当前页面足够大，放得下：
        buffers[0][..8].copy_from_slice(&sec_bytes);
        buffers[0][8..16].copy_from_slice(&usec_bytes);
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");

    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
