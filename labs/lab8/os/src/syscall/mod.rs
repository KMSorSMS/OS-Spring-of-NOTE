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

/// openat syscall
pub const SYSCALL_OPENAT: usize = 56;
/// close syscall
pub const SYSCALL_CLOSE: usize = 57;
/// read syscall
pub const SYSCALL_READ: usize = 63;
/// write syscall
pub const SYSCALL_WRITE: usize = 64;
/// unlinkat syscall
pub const SYSCALL_UNLINKAT: usize = 35;
/// linkat syscall
pub const SYSCALL_LINKAT: usize = 37;
/// fstat syscall
pub const SYSCALL_FSTAT: usize = 80;
/// exit syscall
pub const SYSCALL_EXIT: usize = 93;
/// sleep syscall
pub const SYSCALL_SLEEP: usize = 101;
/// yield syscall
pub const SYSCALL_YIELD: usize = 124;
/// kill syscall
pub const SYSCALL_KILL: usize = 129;
/*
/// sigaction syscall
pub const SYSCALL_SIGACTION: usize = 134;
/// sigprocmask syscall
pub const SYSCALL_SIGPROCMASK: usize = 135;
/// sigreturn syscall
pub const SYSCALL_SIGRETURN: usize = 139;
*/
/// gettimeofday syscall
pub const SYSCALL_GETTIMEOFDAY: usize = 169;
/// getpid syscall
pub const SYSCALL_GETPID: usize = 172;
/// gettid syscall
pub const SYSCALL_GETTID: usize = 178;
/// fork syscall
pub const SYSCALL_FORK: usize = 220;
/// exec syscall
pub const SYSCALL_EXEC: usize = 221;
/// waitpid syscall
pub const SYSCALL_WAITPID: usize = 260;
/// set priority syscall
pub const SYSCALL_SET_PRIORITY: usize = 140;
/*
/// sbrk syscall
pub const SYSCALL_SBRK: usize = 214;
*/
/// munmap syscall
pub const SYSCALL_MUNMAP: usize = 215;
/// mmap syscall
pub const SYSCALL_MMAP: usize = 222;
/// spawn syscall
pub const SYSCALL_SPAWN: usize = 400;
/*
/// mail read syscall
pub const SYSCALL_MAIL_READ: usize = 401;
/// mail write syscall
pub const SYSCALL_MAIL_WRITE: usize = 402;
*/
/// dup syscall
pub const SYSCALL_DUP: usize = 24;
/// pipe syscall
pub const SYSCALL_PIPE: usize = 59;
/// task info syscall
pub const SYSCALL_TASK_INFO: usize = 410;
/// thread_create syscall
pub const SYSCALL_THREAD_CREATE: usize = 460;
/// waittid syscall
pub const SYSCALL_WAITTID: usize = 462;
/// mutex_create syscall
pub const SYSCALL_MUTEX_CREATE: usize = 463;
/// mutex_lock syscall
pub const SYSCALL_MUTEX_LOCK: usize = 464;
/// mutex_unlock syscall
pub const SYSCALL_MUTEX_UNLOCK: usize = 466;
/// semaphore_create syscall
pub const SYSCALL_SEMAPHORE_CREATE: usize = 467;
/// semaphore_up syscall
pub const SYSCALL_SEMAPHORE_UP: usize = 468;
/// enable deadlock detect syscall
pub const SYSCALL_ENABLE_DEADLOCK_DETECT: usize = 469;
/// semaphore_down syscall
pub const SYSCALL_SEMAPHORE_DOWN: usize = 470;
/// condvar_create syscall
pub const SYSCALL_CONDVAR_CREATE: usize = 471;
/// condvar_signal syscall
pub const SYSCALL_CONDVAR_SIGNAL: usize = 472;
/// condvar_wait syscallca
pub const SYSCALL_CONDVAR_WAIT: usize = 473;

mod fs;
mod process;
mod sync;
mod thread;
mod deadlock;
 
use alloc::collections::BTreeMap;
use fs::*;
use process::*;
use sync::*;
use thread::*;

use crate::{fs::Stat, sync::UPSafeCell, task::DynamicMatrix};

use lazy_static::*;

lazy_static! {
    ///创建一个全局变量BtreeMap,用于存放进程pid对应是否开启死锁
    pub static ref ENABLE_DEADLOCK: UPSafeCell<BTreeMap<usize, bool>> =
        unsafe { UPSafeCell::new(BTreeMap::new()) };


    ///创建信号量的Available向量，这里每个Available向量是和一个进程绑定的,这个矩阵就是一个1*m的（1对应进程，m为资源数）
    pub static ref SEM_AVAILABLE: UPSafeCell<BTreeMap<usize,DynamicMatrix<usize>>> =
    unsafe{
        UPSafeCell::new(BTreeMap::new())
    };
    /// 创建一个信号量的分配矩阵Allocation:表示每类资源已分配给每个线程的资源数
    /// Allocation和进程绑定，为n*m矩阵(n对应线程，m对应资源类型数)
    pub static ref SEM_ALLOCATION: UPSafeCell<BTreeMap<usize,DynamicMatrix<usize>>> =
    unsafe{
        UPSafeCell::new(BTreeMap::new())
    };
    /// 信号量的需求矩阵 Need：表示每个线程还需要的各类资源数量。
    /// NEED和进程绑定，为n*m矩阵（n对应线程，m对应资源类型）
    pub static ref SEM_NEED: UPSafeCell<BTreeMap<usize,DynamicMatrix<usize>>> =
    unsafe{
        UPSafeCell::new(BTreeMap::new())
    };


    ///创建互斥量的Available向量，这里每个Available向量是和一个进程绑定的,这个矩阵就是一个1*m的（1对应进程，m为资源数）
    pub static ref MUTX_AVAILABLE: UPSafeCell<BTreeMap<usize,DynamicMatrix<usize>>> =
    unsafe{
        UPSafeCell::new(BTreeMap::new())
    };
    /// 创建一个互斥量的分配矩阵Allocation:表示每类资源已分配给每个线程的资源数
    /// Allocation和进程绑定，为n*m矩阵(n对应线程，m对应资源类型数)
    pub static ref MUTX_ALLOCATION: UPSafeCell<BTreeMap<usize,DynamicMatrix<usize>>> =
    unsafe{
        UPSafeCell::new(BTreeMap::new())
    };
    /// 互斥量的需求矩阵 Need：表示每个线程还需要的各类资源数量。
    /// NEED和进程绑定，为n*m矩阵（n对应线程，m对应资源类型）
    pub static ref MUTX_NEED: UPSafeCell<BTreeMap<usize,DynamicMatrix<usize>>> =
    unsafe{
        UPSafeCell::new(BTreeMap::new())
    };
}


/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 4]) -> isize {
    match syscall_id {
        SYSCALL_DUP => sys_dup(args[0]),
        SYSCALL_LINKAT => sys_linkat(args[1] as *const u8, args[3] as *const u8),
        SYSCALL_UNLINKAT => sys_unlinkat(args[1] as *const u8),
        SYSCALL_OPENAT => sys_open(args[1] as *const u8, args[2] as u32),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_PIPE => sys_pipe(args[0] as *mut usize),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_FSTAT => sys_fstat(args[0], args[1] as *mut Stat),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_SLEEP => sys_sleep(args[0]),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_GETTID => sys_gettid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8, args[1] as *const usize),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_GETTIMEOFDAY => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_SET_PRIORITY => sys_set_priority(args[0] as isize),
        SYSCALL_TASK_INFO => sys_task_info(args[0] as *mut TaskInfo),
        SYSCALL_SPAWN => sys_spawn(args[0] as *const u8),
        SYSCALL_THREAD_CREATE => sys_thread_create(args[0], args[1]),
        SYSCALL_WAITTID => sys_waittid(args[0]) as isize,
        SYSCALL_MUTEX_CREATE => sys_mutex_create(args[0] == 1),
        SYSCALL_MUTEX_LOCK => sys_mutex_lock(args[0]),
        SYSCALL_MUTEX_UNLOCK => sys_mutex_unlock(args[0]),
        SYSCALL_SEMAPHORE_CREATE => sys_semaphore_create(args[0]),
        SYSCALL_SEMAPHORE_UP => sys_semaphore_up(args[0]),
        SYSCALL_ENABLE_DEADLOCK_DETECT => sys_enable_deadlock_detect(args[0]),
        SYSCALL_SEMAPHORE_DOWN => sys_semaphore_down(args[0]),
        SYSCALL_CONDVAR_CREATE => sys_condvar_create(),
        SYSCALL_CONDVAR_SIGNAL => sys_condvar_signal(args[0]),
        SYSCALL_CONDVAR_WAIT => sys_condvar_wait(args[0], args[1]),
        SYSCALL_KILL => sys_kill(args[0], args[1] as u32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
