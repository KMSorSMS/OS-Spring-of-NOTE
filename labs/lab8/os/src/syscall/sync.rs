use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::syscall::deadlock::{init_rsc, update_rsc_down, update_rsc_up};
use crate::syscall::{
    ENABLE_DEADLOCK, MUTX_ALLOCATION, MUTX_AVAILABLE, MUTX_NEED, SEM_ALLOCATION, SEM_AVAILABLE,
    SEM_NEED,
};
use crate::task::{block_current_and_run_next, current_process, current_task, DynamicMatrix};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;

/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    let id_result: isize;
    let pid = current_task().unwrap().process.upgrade().unwrap().getpid();
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!("kernel:pid[{}] tid[{}] sys_mutex_create", pid, tid);
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        id_result = id as isize;
    } else {
        process_inner.mutex_list.push(mutex);
        id_result = process_inner.mutex_list.len() as isize - 1;
    }
    let binding = ENABLE_DEADLOCK.exclusive_access();
    let enabled = binding.get(&pid);
    if let Some(enabled) = enabled {
        if *enabled {
            //在这里，创建锁成功了，为了增加死锁检测算法，这里需要做记录
            let mut binding = MUTX_AVAILABLE.exclusive_access();
            let available = binding.entry(pid).or_insert(DynamicMatrix::new());
            init_rsc(available, id_result as usize, 1);
        }
    }
    id_result
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let pid = current_task().unwrap().process.upgrade().unwrap().getpid();
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!("kernel:pid[{}] tid[{}] sys_mutex_lock", pid, tid);
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    //死锁检测：通过pid得到是否开启死锁检测
    let binding = ENABLE_DEADLOCK.exclusive_access();
    let enabled = binding.get(&pid);
    if let Some(enabled) = enabled {
        if *enabled {
            drop(binding);
            //更新资源信息,同时检测死锁
            //获得三个重要的表：available,allocation,need
            let mut binding1 = MUTX_AVAILABLE.exclusive_access();
            let available = binding1.entry(pid).or_insert(DynamicMatrix::new());
            let mut binding2 = MUTX_ALLOCATION.exclusive_access();
            let allocation = binding2.entry(pid).or_insert(DynamicMatrix::new());
            let mut binding3 = MUTX_NEED.exclusive_access();
            let need = binding3.entry(pid).or_insert(DynamicMatrix::new());
            if update_rsc_down(mutex.is_need(), need, allocation, available, tid, mutex_id) {
                return -0xDEAD;
            } else {
                drop(binding1);
                drop(binding2);
                drop(binding3);
                mutex.lock();
            }
        }
    } else {
        drop(binding);
        mutex.lock();
    }
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let pid = current_task().unwrap().process.upgrade().unwrap().getpid();
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
       pid,tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    //死锁检测：通过pid得到是否开启死锁检测
    let binding = ENABLE_DEADLOCK.exclusive_access();
    let enabled = binding.get(&pid);
    if let Some(enabled) = enabled {
        if *enabled {
            //更新资源信息
            //获得两个重要的表：available和alloc
            let mut binding = MUTX_AVAILABLE.exclusive_access();
            let available = binding.entry(pid).or_insert(DynamicMatrix::new());
            let mut binding = MUTX_ALLOCATION.exclusive_access();
            let allocation = binding.entry(pid).or_insert(DynamicMatrix::new());
            update_rsc_up(allocation, available, tid, mutex_id);
        }
    }
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    let pid = current_task().unwrap().process.upgrade().unwrap().getpid();
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!("kernel:pid[{}] tid[{}] sys_semaphore_create", pid, tid);
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };

    //判定是否要死锁检测
    let binding = ENABLE_DEADLOCK.exclusive_access();
    let enabled = binding.get(&pid);
    if let Some(enabled) = enabled {
        if *enabled {
            //初始化死锁检测信息
            //找到available表，注意考虑表可能不存在
            let mut binding = SEM_AVAILABLE.exclusive_access();
            let available = binding.entry(pid).or_insert(DynamicMatrix::new());
            init_rsc(available, id, res_count);
            println!("\n--id is {} pid is {} num in avi is {}",id,pid,available.get(0, id));
        }
    }
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let pid = current_task().unwrap().process.upgrade().unwrap().getpid();
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!("kernel:pid[{}] tid[{}] sys_semaphore_up", pid, tid);
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    //死锁检测：通过pid得到是否开启死锁检测
    let binding = ENABLE_DEADLOCK.exclusive_access();
    let enabled = binding.get(&pid);
    if let Some(enabled) = enabled {
        if *enabled {
            //更新资源信息
            //获得2个重要的表：available和alloc
            let mut binding1 = SEM_AVAILABLE.exclusive_access();
            let available = binding1.entry(pid).or_insert(DynamicMatrix::new());
            let mut binding2 = SEM_ALLOCATION.exclusive_access();
            let allocation = binding2.entry(pid).or_insert(DynamicMatrix::new());
            //更新资源信息
            update_rsc_up(allocation, available, tid, sem_id as usize);
            drop(binding1);
            drop(binding2);
        }
    }
    drop(binding);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let pid = current_task().unwrap().process.upgrade().unwrap().getpid();
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!("kernel:pid[{}] tid[{}] sys_semaphore_down", pid, tid);
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    //死锁检测：通过pid得到是否开启死锁检测
    let binding = ENABLE_DEADLOCK.exclusive_access();
    let enabled = binding.get(&pid);
    if let Some(enabled) = enabled {
        if *enabled {
            drop(binding);
            //更新资源信息,同时检测死锁
            //获得三个重要的表：available,allocation,need
            let mut binding1 = SEM_AVAILABLE.exclusive_access();
            let available = binding1.entry(pid).or_insert(DynamicMatrix::new());
            let mut binding2 = SEM_ALLOCATION.exclusive_access();
            // println!("--****num is {}--",binding2.get_mut(&pid).unwrap().get(0, sem_id));
            let allocation = binding2.entry(pid).or_insert(DynamicMatrix::new());
            let mut binding3 = SEM_NEED.exclusive_access();
            let need = binding3.entry(pid).or_insert(DynamicMatrix::new());
            print!("\n--available num is {} pid is {}--\n",available.get(0, sem_id),pid);
            if update_rsc_down(
                sem.is_need(),
                need,
                allocation,
                available,
                tid,
                sem_id as usize,
            ) {
                return -0xDEAD;
            } else {
                drop(binding1);
                drop(binding2);
                drop(binding3);
                sem.down();
            }
        } else {
            drop(binding);
            sem.down();
        }
    } else {
        drop(binding);
        sem.down();
    }
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    //这里只是记录需要开启死锁检测这个信息,可以将mutex和semaphore的检测实现合并
    //因为锁可以看做资源为1的信号量
    //获取当前的pcb_id
    let pid = current_task().unwrap().process.upgrade().unwrap().getpid();
    ENABLE_DEADLOCK
        .exclusive_access()
        .insert(pid, _enabled != 0);
    0
}
