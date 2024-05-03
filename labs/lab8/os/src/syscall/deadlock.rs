use alloc::vec;

use crate::task::DynamicMatrix;

/// 在执行p操作的时候更新对线程资源的状态,并且返回是否有死锁
/// 参数是三张关键的表(need,alloc,avai)和是否是改动Need,以及行列号
/// 如果有死锁，返回true
/// 根据是否需要等待资源分成两种情况更新：
/// 更新Need（需要等待资源）
/// 更新Alloc和Available（不需要等待资源）
/// 然后进行死锁检测
pub fn update_rsc_down(
    is_need: bool,
    need_matrix: &mut DynamicMatrix<usize>,
    alloc_matrix: &mut DynamicMatrix<usize>,
    avai_matrix: &mut DynamicMatrix<usize>,
    row: usize,
    col: usize,
) -> bool {
    let result: bool;
    if is_need {
        //获取旧的当前这个sem的need，用于检测到死锁后恢复
        let old_need = *need_matrix.get(row, col);
        //更新need表
        need_matrix.set(row, col, old_need + 1);
        //死锁检测
        result = sem_detect_dead_lock(need_matrix.clone(), alloc_matrix.clone(), avai_matrix.clone(), col);
        //如果发生死锁，恢复
        if result {
            need_matrix.set(row, col, old_need);
        }
    } else {
        //获取旧的当前这个sem的alloc，用于检测到死锁后恢复
        let old_alloc = *alloc_matrix.get(row, col);
        //更新alloc表
        alloc_matrix.set(row, col, old_alloc + 1);
        //获取旧的当前这个sem的available，用于检测到死锁后恢复
        println!("\n--num in avi before is{}",avai_matrix.get(0, col));
        let old_available = *avai_matrix.get(0, col);
        //更新available表
        avai_matrix.set(0, col, old_available - 1);
        //死锁检测
        println!("\n--num in avi is{}",avai_matrix.get(0, col));
        result = sem_detect_dead_lock(need_matrix.clone(), alloc_matrix.clone(), avai_matrix.clone(), col);
        //如果发生死锁，恢复
        if result {
            alloc_matrix.set(row, col, old_alloc);
            avai_matrix.set(0, col, old_available);
        }
    }
    result
}

///和down对应的，也有up操作（v操作）
/// 参数只需要2张表（alloc和available）以及行列号
/// 将alloc的对应行列指向的资源数减一，available的对应列资源数加一，不需要检测死锁
pub fn update_rsc_up(
    alloc_matrix: &mut DynamicMatrix<usize>,
    avai_matrix: &mut DynamicMatrix<usize>,
    row: usize,
    col: usize,
) {
    //获取旧的当前这个sem的alloc，
    let old_alloc = *alloc_matrix.get(row, col);
    //更新alloc表
    alloc_matrix.set(row, col, old_alloc - 1);
    //获取旧的当前这个sem的available
    let old_available = *avai_matrix.get(0, col);
    //更新available表
    avai_matrix.set(0, col, old_available + 1);
}

/// 信号量死锁检测 返回是否发生死锁
/// 参数需要三张表，这里谨记要clone(我选择在函数里面clone)：need,alloc,work（就是available，还有信号量/互斥量的id（对应col）
pub fn sem_detect_dead_lock(
    mut need: DynamicMatrix<usize>,
    mut alloc: DynamicMatrix<usize>,
    mut work: DynamicMatrix<usize>,
    col: usize,
) -> bool {
    //先clone这三个指针指向的矩阵的内容，并构造finish集合，这里统一起见也构造成一个matrix，用0代表false：
    // let mut need: DynamicMatrix<usize> = need_matrix.clone();
    // let mut alloc: DynamicMatrix<usize> = alloc_matrix.clone();
    // let mut work: DynamicMatrix<usize> = work_matrix.clone();
    let thread_num = alloc.row_count();
    //先根据thread_num构造一个fin矩阵，初始化为0
    let mut fin = vec![0;thread_num];
    //开始执行循环的算法
    loop {
        let mut find = false;
        //找到一个可以满足fin[0][thread_num]对应为0，且need小于等于work的线程
        for row in 0..thread_num {
            if fin[row] == 0usize {
                //对这个线程，要查看目前available的东西能不能把它need的全满足，注意要全满足
                //加入的时候也是把该线程已经被alloc的所有资源加入
                let need_row = need.get_row(row).clone();
                let work_row = work.get_row(0).clone();
                let mut can = true;
                //比较能不能满足
                for (need_rsc,work_rsc) in need_row.iter().zip(work_row.iter()) {
                    if need_rsc > work_rsc {
                        can = false;
                    }
                    //把这个线程的fin设置为0
                }
                if can{
                    //如果能满足，就把这个线程的资源释放
                    for (alloc_rsc,work_rsc) in alloc.get_row(row).clone().iter().zip(work_row.iter()) {
                        work.set(0, col, *work_rsc + *alloc_rsc);
                    }
                    //把这个线程的finish设为1
                    fin[row] = 1;
                    find = true;
                    break;
                }
            }
        }
        if find == false {
            break;
        }
    }
    //遍历fin[0]，如果有0，说明有线程没有满足条件，返回true
    for i in 0..thread_num {
        println!("--the fin the val is{}",fin[i]);
        if fin[i] == 0usize {
            return true;
        }
    }
    false
}

/// 创建信号量的时候调用，用于初始化死锁检测的信息，往available的0行添加一列（列号由分配的col）
/// 传入参数是avail表，列号（也就是sem_id或者mutex_id）
pub fn init_rsc(avai_matrix: &mut DynamicMatrix<usize>, col: usize, res_count: usize) {
    //向aviailable表中添加一列
    avai_matrix.set(0, col, res_count);
    println!("the val is{}",avai_matrix.get(0, col));
}
