//! File and filesystem-related syscalls
use core::mem;


use crate::fs::{open_file, OpenFlags, Stat};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
/// fd: 文件描述符
/// st: 文件状态结构体，结果存放在st中，注意虚拟地址到物理地址的转换
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    //首先要根据fd找到对应的文件，然后将文件的信息填充到st中
    //检验一下不能是标准输出的1和标准输入的0还有标准错误输出的2，这两个不算是文件
    if _fd == 0 || _fd == 1 || _fd == 2 {
        return -1;
    }
    //获取当前任务的tcb
    let token = current_user_token();
    let task = current_task().unwrap();
    //得到inner
    let inner = task.inner_exclusive_access();
    //访问fd号对应的文件描述符，这里应该是OSInode类型
    if let Some(file) = &inner.fd_table[_fd] {
        let file = file.clone();
        //可以drop inner了这里
        drop(inner);
        // //强制转换为OSInode类型 这种实现不好，应该是通过File trait来实现最好
        // let os_inode = Arc::into_raw(file.into()) as *mut OSInode;
        let mut st = translated_byte_buffer(token, _st as *const u8, core::mem::size_of::<Stat>());

        //获取到文件的inode
        let stat = file.stat();
        
        //打成bytes
        let stat_bytes: [u8; mem::size_of::<Stat>()] = unsafe { core::mem::transmute(stat) };
        //将stat的信息填充到st中,和之前的taskinfo处理一样，注意页的split
        if st[0].len() < core::mem::size_of::<Stat>() {
            //说明跨页了
            let len = st[0].len();
            st[0].copy_from_slice(&stat_bytes[..len]);
            //剩下的给到下一页
            st[1][..(mem::size_of::<Stat>() - len)].copy_from_slice(&stat_bytes[len..]);
        } else {
            //没跨页
            st[0][..mem::size_of::<Stat>()].copy_from_slice(&stat_bytes);
        }
        
    }else{
        return -1;
    }
    0
}

/// YOUR JOB: Implement linkat.
/// 参数解释：
/// olddirfd(始终为AT_FDCWD (-100))，newdirfd(始终为AT_FDCWD (-100))，flags(始终为0)均是为了兼容POSIX标准
/// oldpath：原有文件路径、newpath: 新的链接文件路径。
/// 仅存在根目录 / 一个目录
/// 硬链接：两个不同名称目录项指向同一个磁盘块
pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    //借鉴sys_open，我先得为_new_name创建一个文件
    let token = current_user_token();
    // 获取到新文件名
    let new_name = translated_str(token, _new_name);
    //只需要参考文件索引的过程，我们让新文件名指向原文件的inode即可
    //获取到根目录的inode
    let root_inode = crate::fs::ROOT_INODE.clone();
    //根据ROOT_INODE查找到原文件的inode，我们这里和新建一个file的最大区别就是不会真的创建一个inode
    let old_name = translated_str(token, _old_name);
    if let Some(old_inode) = root_inode.find(&old_name.as_str()) {
        //创建一个file，但是inode不是alloc产生，而是由old_inode，从而使得两个文件指向同一个inode
        // 需要转换一下，通过Inode类型找到它的inode_id
        root_inode.create_file_inode(
            &new_name,
            old_inode
                .get_inode_id_by_name(&root_inode, &old_name)
                .unwrap(),
        )
    } else {
        -1
    }
}

/// YOUR JOB: Implement unlinkat.
/// 这里和前面link的流程比较类似，删除操作模仿clear，
/// 但是对于非最后一个文件，我们只是删除索引块，不去清空数据
pub fn sys_unlinkat(_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let name = translated_str(token, _name);
    //因为我们只有根目录，所以这就是文件的所在目录
    let root_inode = crate::fs::ROOT_INODE.clone();
    //找到文件的inode
    if let Some(inode) = root_inode.find(&name.as_str()) {
        //删除文件
        inode.unlink(&root_inode, &name)
    } else {
        -1
    }
}
