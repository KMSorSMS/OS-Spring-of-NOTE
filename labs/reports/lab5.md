# LAB5---袁子为

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > *《你交流的对象说明》*
   >
   > 无交流

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > *《你参考的资料说明》*
   >
   > 仅仅查看了题目代码和rcore文档

\3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

\4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

## 编程作业

写在博客里面了：

[rcore-lab5](https://liamy.clovy.top/article/OS_Tutorial/lab8)

## 简答作业

## 1.

在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 - 需要回收的资源有哪些？ - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？

### 先直接开门见山给出所有问题的答案：

需要回收的与线程相关的资源有：

tid、线程控制块（包含了KernelStack（线程的内核栈）/trap_cx（trap上下文）/TaskUserRes（这里面又包含了用户栈、tid内容）），所以把括号内的东西打散开，也就是释放的所有具体内容

间接联系的还有线程共用的地址空间

除了和线程关联的，这里主线程还会释放作为进程拥有的子进程信息。

其它线程的TaskControlBlock 可能这些地方引用：

1. exit_current_and_run_next里面

   ```rust
           // deallocate user res (including tid/trap_cx/ustack) of all threads
           // it has to be done before we dealloc the whole memory_set
           // otherwise they will be deallocated twice
           let mut recycle_res = Vec::<TaskUserRes>::new();
           for task in process_inner.tasks.iter().filter(|t| t.is_some()) {
               let task = task.as_ref().unwrap();
               // if other tasks are Ready in TaskManager or waiting for a timer to be
               // expired, we should remove them.
               //
               // Mention that we do not need to consider Mutex/Semaphore since they
               // are limited in a single process. Therefore, the blocked tasks are
               // removed when the PCB is deallocated.
               trace!("kernel: exit_current_and_run_next .. remove_inactive_task");
               remove_inactive_task(Arc::clone(&task));
               let mut task_inner = task.inner_exclusive_access();
               if let Some(res) = task_inner.res.take() {
                   recycle_res.push(res);
               }
           }
   ```

   这里肯定是要回收的，这里就是tid等于0的时候，这个进程需要释放，所以要回收线程的所有资源（核心依赖于remove_inactive_task这个函数来回收）

2. 在`sys_waittid` 等待指定的task的时候，取出了其它线程的task，判断是否它exit了，如果是就需要释放进程管理这边占用的task tid的资源，只是释放一个id号其实，不过也是需要释放资源的，如果wait的task（线程）没有exit，那么就不会释放资源，这部分代码如下：

   ```rust
   let mut exit_code: Option<i32> = None;
       let waited_task = process_inner.tasks[tid].as_ref();
       if let Some(waited_task) = waited_task {
           if let Some(waited_exit_code) = waited_task.inner_exclusive_access().exit_code {
               exit_code = Some(waited_exit_code);
           }
       } else {
           // waited thread does not exist
           return -1;
       }
       if let Some(exit_code) = exit_code {
           // dealloc the exited thread
           process_inner.tasks[tid] = None;
           exit_code
       } else {
           // waited thread has not exited
           -2
       }
   ```

看起来，我是根据 process_inner.tasks这个变量全局搜索看的，（process_inner）这个局部变量的取名我检查了一下应该都是一致的，就是进程的inner属性，这里面存的有进程包含的所有线程，所以要拿到其它线程，肯定要经过这一层取inner的操作，全局搜索下来就这两个地方。

### 下面是分析：

我觉得主要是参考tutorial和sys_waittid和exit_current_and_run_next这两个函数的实现：

**综述**

一般情况下进程/主线程要负责通过 `waittid` 来等待它创建出来的线程（不是主线程）结束并回收它们在内核中的资源 （如线程的内核栈、线程控制块等）。如果进程/主线程先调用了 `exit` 系统调用来退出，那么整个进程 （包括所属的所有线程）都会退出，而对应父进程会通过 `waitpid` 回收子进程剩余还没被回收的资源。

**先说waittid（有一种规范的操作就是用waittid来等待所有线程结束）**

在sys_waittid的实现里面，代码是：

```rust
/// wait for a thread to exit syscall
///
/// thread does not exist, return -1
/// thread has not exited yet, return -2
/// otherwise, return thread's exit code
pub fn sys_waittid(tid: usize) -> i32 {
    let task = current_task().unwrap();
    let process = task.process.upgrade().unwrap();
    let task_inner = task.inner_exclusive_access();
    let mut process_inner = process.inner_exclusive_access();
    // a thread cannot wait for itself
    if task_inner.res.as_ref().unwrap().tid == tid {
        return -1;
    }
    let mut exit_code: Option<i32> = None;
    let waited_task = process_inner.tasks[tid].as_ref();
    if let Some(waited_task) = waited_task {
        if let Some(waited_exit_code) = waited_task.inner_exclusive_access().exit_code {
            exit_code = Some(waited_exit_code);
        }
    } else {
        // waited thread does not exist
        return -1;
    }
    if let Some(exit_code) = exit_code {
        // dealloc the exited thread
        process_inner.tasks[tid] = None;
        exit_code
    } else {
        // waited thread has not exited
        -2
    }
}
```

我们的主线程退出前，会主动调用这个函数释放掉该进程管理的所有线程（通过比如for循环依次传入该进程的所有tid编号）

这里关键点释放是在：

```
process_inner.tasks[tid] = None; exit_code
```

由于线程0会正常情况退出会采用这种方式，所以会循环的释放掉所有的在该进程记录下的taskcontrolblock（这个同时也是释放了tid），其它资源如内核栈，是在线程自身调用exit的时候回收的。不过哪怕是这样，最后thread 0这个线程释放的时候，还是会去调用exit然后使用到我们exit_current_and_run_next，只是说对于线程资源回收的部分是没有的，实际的回收是在这个waittid

**直接退出，没有waittid完全**

而对于直接的线程0退出，没有去使用waittid，那么核心就在exit_current_and_run_next的这部分代码：

```rust
// however, if this is the main thread of current process
    // the process should terminate at once
    if tid == 0 {
        let pid = process.getpid();
        if pid == IDLE_PID {
            println!(
                "[kernel] Idle process exit with exit_code {} ...",
                exit_code
            );
            if exit_code != 0 {
                //crate::sbi::shutdown(255); //255 == -1 for err hint
                crate::board::QEMU_EXIT_HANDLE.exit_failure();
            } else {
                //crate::sbi::shutdown(0); //0 for success hint
                crate::board::QEMU_EXIT_HANDLE.exit_success();
            }
        }
        remove_from_pid2process(pid);
        let mut process_inner = process.inner_exclusive_access();
        // mark this process as a zombie process
        //把当前进程标记为僵尸进程，设置进程状态为"僵尸"是为了表示该进程已经终止。
        //当一个进程终止时，它的资源（如内存、文件描述符等）需要被释放，
        //并且父进程需要知道该进程的退出状态。将进程状态设置为"僵尸"是一种通用的做法，
        //用于表示进程已经终止但尚未被其父进程回收。
        process_inner.is_zombie = true;
        // record exit code of main process
        process_inner.exit_code = exit_code;

        {
            // move all child processes under init process
            let mut initproc_inner = INITPROC.inner_exclusive_access();
            for child in process_inner.children.iter() {
                child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }

        // deallocate user res (including tid/trap_cx/ustack) of all threads
        // it has to be done before we dealloc the whole memory_set
        // otherwise they will be deallocated twice
        let mut recycle_res = Vec::<TaskUserRes>::new();
        for task in process_inner.tasks.iter().filter(|t| t.is_some()) {
            let task = task.as_ref().unwrap();
            // if other tasks are Ready in TaskManager or waiting for a timer to be
            // expired, we should remove them.
            //
            // Mention that we do not need to consider Mutex/Semaphore since they
            // are limited in a single process. Therefore, the blocked tasks are
            // removed when the PCB is deallocated.
            trace!("kernel: exit_current_and_run_next .. remove_inactive_task");
            remove_inactive_task(Arc::clone(&task));
            let mut task_inner = task.inner_exclusive_access();
            if let Some(res) = task_inner.res.take() {
                recycle_res.push(res);
            }
        }
        // dealloc_tid and dealloc_user_res require access to PCB inner, so we
        // need to collect those user res first, then release process_inner
        // for now to avoid deadlock/double borrow problem.
        drop(process_inner);
        recycle_res.clear();

        let mut process_inner = process.inner_exclusive_access();
        process_inner.children.clear();
        // deallocate other data in user space i.e. program code/data section
        process_inner.memory_set.recycle_data_pages();
        // drop file descriptors
        process_inner.fd_table.clear();
        // remove all tasks
        process_inner.tasks.clear();
    }
```

鉴于代码很长，但是的确关键的释放内容在代码里，我就往代码里面添加注释来详细说明☝️

算了，感觉这样写不美观，我摘抄出来一节节说：

```rust
    		 //把当前进程标记为僵尸进程，设置进程状态为"僵尸"是为了表示该进程已经终止。
        //当一个进程终止时，它的资源（如内存、文件描述符等）需要被释放，
        //并且父进程需要知道该进程的退出状态。将进程状态设置为"僵尸"是一种通用的做法，
        //用于表示进程已经终止但尚未被其父进程回收。
        process_inner.is_zombie = true;
```

僵尸进程的处理是很常见的（在linux上）：

<aside> <img src="/icons/cursor-click_purple.svg" alt="/icons/cursor-click_purple.svg" width="40px" /> 任何一个子进程(init除外)在exit()之后，并非马上就消失掉，而是留下一个称为僵尸进程(Zombie)的数据结构，等待父进程处理。这是每个 子进程在结束时都要经过的阶段。如果子进程在exit()之后，父进程没有来得及处理，这时用ps命令就能看到子进程的状态是“Z”。如果父进程能及时 处理，可能用ps命令就来不及看到子进程的僵尸状态，但这并不等于子进程不经过僵尸状态。 如果父进程在子进程结束之前退出，则子进程将由init接管。init将会以父进程的身份对僵尸状态的子进程进行处理。 一个进程如果只复制fork子进程而不负责对子进程进行wait()或是waitpid()调用来释放其所占有资源的话，那么就会产生很多的僵死进程，如果要消灭系统中大量的僵死进程，只需要将其父进程杀死，此时所有的僵死进程就会编程孤儿进程，从而被init所收养，这样init就会释放所有的僵死进程所占有的资源，从而结束僵死进程。

———from https://zhuanlan.zhihu.com/p/349109411

</aside>

所以下面你可以看到rcore这里写的：

```rust
        {
            // move all child processes under init process
            let mut initproc_inner = INITPROC.inner_exclusive_access();
            for child in process_inner.children.iter() {
                child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }
```

删除任务控制块(taskcontrolblock)：

```rust
           	let task = task.as_ref().unwrap();
            // if other tasks are Ready in TaskManager or waiting for a timer to be
            // expired, we should remove them.
            //
            // Mention that we do not need to consider Mutex/Semaphore since they
            // are limited in a single process. Therefore, the blocked tasks are
            // removed when the PCB is deallocated.
            trace!("kernel: exit_current_and_run_next .. remove_inactive_task");
            remove_inactive_task(Arc::clone(&task));
            let mut task_inner = task.inner_exclusive_access();
            if let Some(res) = task_inner.res.take() {
                recycle_res.push(res);
            }
```

释放进程相关的资源（地址空间、子进程的记录、文件系统的资源、以及清除在进程里面记录的线程列表）

```rust
	      process_inner.children.clear();
        // deallocate other data in user space i.e. program code/data section
        process_inner.memory_set.recycle_data_pages();
        // drop file descriptors
        process_inner.fd_table.clear();
        // remove all tasks
        process_inner.tasks.clear();
```

## 2.

对比以下两种 `Mutex.unlock` 的实现，二者有什么区别？这些区别可能会导致什么问题？

### 代码

```rust
 1 impl Mutex for Mutex1 {
 2    fn unlock(&self) {
 3        let mut mutex_inner = self.inner.exclusive_access();
 4        assert!(mutex_inner.locked);
 5        mutex_inner.locked = false;
 6        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
 7            add_task(waking_task);
 8        }
 9    }
10 }
11
12 impl Mutex for Mutex2 {
13    fn unlock(&self) {
14        let mut mutex_inner = self.inner.exclusive_access();
15        assert!(mutex_inner.locked);
16        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
17            add_task(waking_task);
18        } else {
19            mutex_inner.locked = false;
20        }
21    }
22 }
```

### Mutex2

Mutex2就是我们实验代码里面的MutexBlocking的实现

我先解释它，（文档里面其实已经说的很清楚了）：

```
16 if let Some(waking_task) = mutex_inner.wait_queue.pop_front() { 17            add_task(waking_task); 18        } else { 19            mutex_inner.locked = false; 20        }
```

- 16-17行 如果有等待的线程，唤醒等待最久的那个线程，相当于将锁的所有权移交给该线程。
- 而如果没有，就在19行的位置释放锁

### ok，那我们再看看Mutex1和它有哪里不同（这里先给出答案，Mutex1是有大问题的）

最明显的就是Mutex1一来就先释放了锁，并且没有任何判定的执行唤醒任务的功能（然而两个操作只应该是一次解锁对应一个），我这里直接给出一个会产生问题的例子（Mutex1）：

假如现在有三个线程（主线程0，其它线程1，其它线程2），主线程拿到锁，另外其它线程1等待锁，而其它线程2正常运行（后续会需要锁，目前还没有）。

1. 主线程释放锁，此时线程2没有等锁（正常运行），线程1等待锁（阻塞）
2. 主线程释放锁（目前锁为空的，无人占用）
3. 主线程执行到第7行（因为1等待锁，所以wait_queue中可以弹出一个需要唤醒的线程1），然后线程1被唤醒，得到互斥锁，执行临界区内容
4. 这个时候，如果2执行到临界区，想要获得锁（此时线程1还在临界区），是能够得到的，因为锁是空的，所以会虽然使用了互斥锁，但是线程1、2同时进入了临界区，并没有满足最基本的互斥锁的要求： 互斥性（mutual exclusion），即锁是否能够有效阻止多个线程进入临界区，这是最基本的属性。

### 补充

不过对于Mutex1，其实也是有一点小问题，参考rocore里面的代码，这里没有更新任务的状态为Ready，使得任务仍然为block，这一点是不太好的