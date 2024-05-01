//! Process management syscalls
use core::mem::{self, size_of};

use alloc::sync::Arc;

use crate::{
    config::MAX_SYSCALL_NUM, loader::get_app_data_by_name, mm::{translated_byte_buffer, translated_refmut, translated_str, MapPermission, VirtAddr, VirtPageNum}, syscall::TASK_INFO_LIST, task::{
        add_task, current_task, current_user_token, exit_current_and_run_next, suspend_current_and_run_next, TaskControlBlock, TaskStatus
    }, timer::get_time_us
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
    /// Create a new TaskInfo
    pub fn new() -> Self {
        TaskInfo {
            status: TaskStatus::Ready,
            syscall_times: [0; MAX_SYSCALL_NUM],
            time: 0,
        }
    }
    /// Set task status
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }
    /// Add syscall times
    pub fn add_syscall_time(&mut self, syscall: usize) {
        self.syscall_times[syscall] += 1;
        // if syscall == 169 && self.syscall_times[syscall] > 18000{
        //     println!("\n--{}--\n", self.syscall_times[syscall])
        // }
    }
    /// Set total running time
    pub fn set_time(&mut self, task_start: usize, task_syscall: usize) {
        self.time = task_syscall - task_start;
    }
    // ///实现将TaskInfo里面的所有信息转为u8的数组
    // pub fn to_bytes(&self) -> Vec<u8> {
    //     let mut ret_bytes: Vec<u8> = vec![];
    //     ret_bytes.extend_from_slice(&(self.status as usize).to_le_bytes());
    //     for &num in self.syscall_times.iter() {
    //         ret_bytes.extend_from_slice(&num.to_le_bytes());
    //     }
    //     ret_bytes.extend_from_slice(&self.time.to_le_bytes());
    //     ret_bytes
    // }
}
/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_get_time NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
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
        buffers[1][..(8 - len)].copy_from_slice(&sec_bytes[len..]);
        //后续的可以直接放到下一页
        buffers[1][(8 - len)..].copy_from_slice(&usec_bytes);
    } else if buffers[0].len() < 16 {
        //走到这里是当前页剩余超过8字节，但是少于16字节，导致usec放的时候会跨页
        buffers[0][..8].copy_from_slice(&sec_bytes);
        // 当前页能放置usec的长度就是buffer[0]的长度减去sec_bytes的长度（8）
        let len = buffers[0].len() - 8;
        buffers[0][8..].copy_from_slice(&usec_bytes[..len]);
        buffers[1][..(8 - len)].copy_from_slice(&usec_bytes[len..]);
    } else {
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
    trace!(
        "kernel:pid[{}] sys_task_info NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
     //这里的实现和之前一样，并且和sys_get_time的更新实现也是类似：
    //取出当前任务的taskinfo
    // 取出当前任务的taskinfo，然后一个个传给指针
    let current = current_task().unwrap().pid.0;
    let task_info_list = TASK_INFO_LIST.exclusive_access();
    let current_task_info = task_info_list.get(&current).unwrap();
    let current_task_info_byte: [u8; mem::size_of::<TaskInfo>()] =
        unsafe { mem::transmute(*current_task_info) };
    // println!(
    //     "\nthe syscall time of time{}---\n",
    //     current_task_info.syscall_times[169]
    // );
    // println!(
    //     "\nthe syscall time of taskinfo{}---\n",
    //     current_task_info.syscall_times[410]
    // );
    // println!(
    //     "\nthe syscall time of write{}---\n",
    //     current_task_info.syscall_times[64]
    // );
    //这里甚至比sys_get_time简单，只需要传入一个数据，
    let mut buffers = translated_byte_buffer(
        current_user_token(),
        _ti as *const u8,
        size_of::<TaskInfo>(),
    );
    //检查是否split了
    if buffers[0].len() < size_of::<TaskInfo>() {
        //说明跨页了
        let len = buffers[0].len();
        buffers[0].copy_from_slice(&current_task_info_byte[..len]);
        //剩下的给到下一页
        buffers[1][..(size_of::<TaskInfo>() - len)].copy_from_slice(&current_task_info_byte[len..]);
    } else {
        //没跨页
        buffers[0][..size_of::<TaskInfo>()].copy_from_slice(&current_task_info_byte);
    }
    0
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_mmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    //这里可以参考task.rs里面给任务分配内存（栈，trapcontext）的操作
    //先把参数检查做了，要求有：
    /*
    start 需要映射的虚存起始地址，要求按页对齐
    len 映射字节长度，可以为 0
    port：第 0 位表示是否可读，第 1 位表示是否可写，第 2 位表示是否可执行。其他位无效且必须为 0
     */
    //先检查start是否对齐
    if _start % 4096 != 0 {
        return -1;
    }
    if _port & 0b111 != _port || _port & 0b111 == 0 {
        return -1;
    }
    if _len == 0 {
        return 0;
    }
    //检查是否已经存在页映射了[start,start+len)这段虚拟地址
    //获取当前任务的innertaskcontrolblock,千万要记得drop
    let binding = current_task().unwrap();
    let mut tcb = binding.inner_exclusive_access();
    //找到start对应的vpn号以及start+len对应的vpn号
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + _len);
    let start_vpn: VirtPageNum = start_va.floor();
    let end_vpn: VirtPageNum = end_va.ceil();
    //检查是否已经存在页映射了[start,start+len)这段虚拟地址,检查
    if tcb.memory_set.check_overlap(start_vpn,end_vpn){
        return -1;
    }
    //完成了参数检查，接下来就是分配内存了,先把MapPermission由port确定了
    let mut map_perm = MapPermission::U;
    if _port & 0b001 != 0 {
        map_perm |= MapPermission::R;
    }
    if _port & 0b010 != 0 {
        map_perm |= MapPermission::W;
    }
    if _port & 0b100 != 0 {
        map_perm |= MapPermission::X;
    }
    //然后根据地址先插入我们的memory_set,这一步就完成了映射
    //这里有个边界问题，比如len为4096，这个时候我们只需要分配一个整页，但是end_va取了ceil
    tcb.memory_set
        .insert_framed_area(start_va, end_va, map_perm);
    // //接下来就是获取物理地址，然后映射到虚拟地址，写到tcb的pagetable里面，需要分配不止一个frame
    // for vpn in start_vpn.0..end_vpn.0{
    //     if let Some(phy_frame_tracker) = frame_alloc(){
    //         //分配成功,FrameTracker入到pagetable里面
    //         tcb.memory_set.map2physical(vpn.into(), phy_frame_tracker.ppn, map_perm);
    //     }else {
    //         //分配失败
    //         return -1;
    //     }
    // }
    drop(tcb);
    0
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_munmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    //取消[start,start+len)这段虚拟地址的映射,同样要参数检验
    //先检查start是否对齐
    if _start % 4096 != 0 {
        return -1;
    }
    if _len == 0 {
        return 0;
    }
    //检查是否已经存在页映射了[start,start+len)这段虚拟地址，如果没有，直接返回-1
    //获取当前任务的taskcontrolblock
    let binding = current_task().unwrap();
    let mut tcb = binding.inner_exclusive_access();
    //找到start对应的vpn号以及start+len对应的vpn号
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + _len);
    let start_vpn: VirtPageNum = start_va.floor();
    let end_vpn: VirtPageNum = end_va.ceil();
    //检查是否已经存在页映射了[start,start+len)这段虚拟地址,需要完全的覆盖，不能只是检查有交集，需要查页表,检查
    for vpn in start_vpn.0..end_vpn.0{
        if let Some(pet) = tcb.memory_set.translate(vpn.into()){
            if !pet.is_valid(){
                return -1;
            }
        }else{
            return -1;
        }
    }
    //完成了参数检查，接下来就是删除内存了,已经包装好了方法unmap
    tcb.memory_set.unmap(start_vpn, end_vpn);
    0
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
/// 注意返回的是pid
pub fn sys_spawn(_path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    //这里仿照sys_exec的实现，不要去复制父进程的地址空间，和fork区别开,但是fork做的其它内容都要做
    //先得数据：
    let token = current_user_token();
    let path = translated_str(token, _path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        //创建一个新的task，这个过程我包装在了task.rs里面
        let new_task:Arc<TaskControlBlock> = TaskControlBlock::new(&data).into();
        //这里需要像fork一样，把新创建的进程作为child加入到当前进程的children里面
        let binding = current_task().unwrap();
        let mut parent_inner = binding.inner_exclusive_access();
        parent_inner.children.push(new_task.clone());
        //然后把新的task加入到scheduler里面
        let newpid = new_task.pid.0;
        add_task(new_task);
        newpid as isize
    } else {
        -1
    }
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}
