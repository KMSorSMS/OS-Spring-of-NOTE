//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    /// 对这个函数进行改动来实现stride调度，选取任务时，选取当前stride最小的任务
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // self.ready_queue.pop_front()
        //如果为空，直接返回None
        if self.ready_queue.is_empty() {
            return None;
        }
        let mut min_stride = isize::MAX;
        let mut min_stride_task_index = 0;
        for i in 0..self.ready_queue.len() {
            let task = self.ready_queue.get(i).unwrap();
            let stride = task.get_stride();
            if stride < min_stride {
                min_stride = stride;
                min_stride_task_index = i;
            }
        }
        //该任务会被执行，那么stride需要更新
        let task = self.ready_queue.get(min_stride_task_index).unwrap();
        task.pass();
        //调用remove取出index指向的待运行任务
        self.ready_queue.remove(min_stride_task_index)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
