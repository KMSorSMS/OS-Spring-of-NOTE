//! Process management syscalls
use core::mem::{self, size_of};

// use alloc::vec;
// use alloc::vec::Vec;
use crate::{
    config::MAX_SYSCALL_NUM,
    mm::{translated_byte_buffer, MapPermission, VirtAddr, VirtPageNum},
    syscall::TASK_INFO_LIST,
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus, TASK_MANAGER,
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
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
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
        // if syscall == 169 && self.syscall_times[syscall] > 18000{
        //     println!("\n--{}--\n", self.syscall_times[syscall])
        // }
    }
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
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    //这里的实现和之前一样，并且和sys_get_time的更新实现也是类似：
    //取出当前任务的taskinfo
    // 取出当前任务的taskinfo，然后一个个传给指针
    let current = TASK_MANAGER.get_current_task();
    let task_info_list = TASK_INFO_LIST.exclusive_access();
    let current_task_info = &task_info_list[current];
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

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
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
    let current = TASK_MANAGER.get_current_task();
    //获取当前任务的taskcontrolblock
    let tcb = TASK_MANAGER.get_task(current);
    //找到start对应的vpn号以及start+len对应的vpn号
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + _len);
    let start_vpn: VirtPageNum = start_va.floor();
    let end_vpn: VirtPageNum = end_va.ceil();
    //检查是否已经存在页映射了[start,start+len)这段虚拟地址,检查
    println!("\n***check overlap");
    if tcb.memory_set.check_overlap(start_vpn,end_vpn){
        println!("\n---***find overlap");
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
    println!("\n***in insert");
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
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    //取消[start,start+len)这段虚拟地址的映射,同样要参数检验
    //先检查start是否对齐
    if _start % 4096 != 0 {
        return -1;
    }
    if _len == 0 {
        return 0;
    }
    //检查是否已经存在页映射了[start,start+len)这段虚拟地址，如果没有，直接返回-1
    let current = TASK_MANAGER.get_current_task();
    //获取当前任务的taskcontrolblock
    let tcb = TASK_MANAGER.get_task(current);
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
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
