# LAB1---袁子为

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

[rcoreLab-03](https://liamy.clovy.top/article/OS_Tutorial/lab3)

## 简答作业

### 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 [三个 bad 测例 (ch2b_bad_*.rs)](https://github.com/LearningOS/rCore-Tutorial-Test-2024S/tree/master/src/bin) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

```makefile

ifeq ($(BASE), 3)
	APPS := $(wildcard $(APP_DIR)/ch2b_bad_*.rs)
endif

```

单独写一个一个把三个bad测试用例拎出来的，可以看到结果：

![image-20240426202934989](https://cdn.jsdelivr.net/gh/KMSorSMS/picGallery@master/img/202404262245827.png)

使用的rustsbi版本是：

RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0

#### 前置知识：

因为我们修改了stvec寄存器：

```rust
global_asm!(include_str!("trap.S"));

/// Initialize trap handling
pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}
```

所以导致了在用户态的应用，当它：

1. 用户态软件为获得内核态操作系统的服务功能而执行特殊指令
2. 在执行某条指令期间产生了错误（如执行了用户态不允许执行的指令或者其他错误）并被 CPU 检测到

会触发从用户态到内核态的异常，而这个异常的处理程序由stvec寄存器写入了我们写的__alltraps()函数地址，然后进入我们编写的程序来在s模式（内核模式）进行处理。

给一个表：（这个表的异常编号和代码里面riscv的异常编号有出入，以代码的为准）

| Interrupt | Exception Code | Description                    |
| --------- | -------------- | ------------------------------ |
| 0         | 0              | Instruction address misaligned |
| 0         | 1              | Instruction access fault       |
| 0         | 2              | Illegal instruction            |
| 0         | 3              | Breakpoint                     |
| 0         | 4              | Load address misaligned        |
| 0         | 5              | Load access fault              |
| 0         | 6              | Store/AMO address misaligned   |
| 0         | 7              | Store/AMO access fault         |
| 0         | 8              | Environment call from U-mode   |
| 0         | 9              | Environment call from S-mode   |
| 0         | 11             | Environment call from M-mode   |
| 0         | 12             | Instruction page fault         |
| 0         | 13             | Load page fault                |
| 0         | 15             | Store/AMO page fault           |

riscv库里面的异常是：

```rust
pub enum Exception {
    InstructionMisaligned,
    InstructionFault,
    IllegalInstruction,
    Breakpoint,
    LoadFault,
    StoreMisaligned,
    StoreFault,
    UserEnvCall,
    VirtualSupervisorEnvCall,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
    InstructionGuestPageFault,
    LoadGuestPageFault,
    VirtualInstruction,
    StoreGuestPageFault,
    Unknown,
}
```

#### 解释内容

##### 第一个bad_address

```rust
pub fn main() -> isize {
    unsafe {
        #[allow(clippy::zero_ptr)]
        (0x0 as *mut u8).write_volatile(0);
    }
    panic!("FAIL: T.T\n");
}
```

这是因为访问地址错误，

具体而言是属于storeFault：

![image-20240426211252214](https://cdn.jsdelivr.net/gh/KMSorSMS/picGallery@master/img/202404262245828.png)

因为试图写入0x0是一个无法访问的内存地址

##### 第二个bad_instructions

```rust
pub fn main() -> ! {
    unsafe {
        core::arch::asm!("sret");
    }
    panic!("FAIL: T.T\n");
}
```

这里是sret这个指令触发的异常：

这里是rcore_tutorial写了的：

> 在 RISC-V 中，会有两类属于高特权级 S 模式的特权指令：
>
> - 指令本身属于高特权级的指令，如 `sret` 指令（表示从 S 模式返回到 U 模式）。
> - 指令访问了 [S模式特权级下才能访问的寄存器](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/4trap-handling.html#term-s-mod-csr) 或内存，如表示S模式系统状态的 **控制状态寄存器** `sstatus` 等。
>
> | 指令                   | 含义                                                         |
> | ---------------------- | ------------------------------------------------------------ |
> | sret                   | 从 S 模式返回 U 模式：在 U 模式下执行会产生非法指令异常      |
> | wfi                    | 处理器在空闲时进入低功耗状态等待中断：在 U 模式下执行会产生非法指令异常 |
> | sfence.vma             | 刷新 TLB 缓存：在 U 模式下执行会产生非法指令异常             |
> | 访问 S 模式 CSR 的指令 | 通过访问 [sepc/stvec/scause/sscartch/stval/sstatus/satp等CSR](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/4trap-handling.html#term-s-mod-csr) 来改变系统状态：在 U 模式下执行会产生非法指令异常 |

##### 同样可以说明第三个

```rust
pub fn main() -> ! {
    let mut sstatus: usize;
    unsafe {
        core::arch::asm!("csrr {}, sstatus", out(reg) sstatus);
    }
    panic!("(-_-) I get sstatus:{:x}\nFAIL: T.T\n", sstatus);
}

```

`core::arch::asm!("csrr {}, sstatus", out(reg) sstatus);`这行代码使用`csrr`指令读取`sstatus`寄存器的值，并将结果存储在`sstatus`变量中。

这里的`sstatus`是RISC-V架构中的一个特殊寄存器，它包含了一些关于当前处理器状态的信息，例如当前的特权级别和一些使能位。

所以在U特权级别访问这个`sstatus`就会触发异常

> **S模式下最重要的 sstatus 寄存器**
>
> 注意 `sstatus` 是 S 特权级最重要的 CSR，可以从多个方面控制 S 特权级的 CPU 行为和执行状态。

### 深入理解 [trap.S](https://github.com/LearningOS/rCore-Tutorial-Code-2024S/blob/ch3/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:

前置一些寄存器的内容：

![image-20240426221911897](https://cdn.jsdelivr.net/gh/KMSorSMS/picGallery@master/img/202404262219319.png)

| CSR 名  | 该 CSR 与 Trap 相关的功能                                    |
| ------- | ------------------------------------------------------------ |
| sstatus | `SPP` 等字段给出 Trap 发生之前 CPU 处在哪个特权级（S/U）等信息 |
| sepc    | 当 Trap 是一个异常的时候，记录 Trap 发生之前执行的最后一条指令的地址 |
| scause  | 描述 Trap 的原因                                             |
| stval   | 给出 Trap 附加信息                                           |
| stvec   | 控制 Trap 处理代码的入口地址                                 |

#### 1、L40：刚进入 `__restore` 时，`a0` 代表了什么值。请指出 `__restore` 的两种使用情景。

刚进入`__restore`,此时a0存储的值有两种可能：

1. 一种是函数trap_handler的返回值是一个trapcontext（是经过了异常处理后返回的context）
2. 另一种是传入switch函数的TaskContext（并且是current的而不是next的）

但是a0的值并没有被__restore用到

<u>**__restore的两种使用情景**</u>

##### **第一种情景**：

是在处理异常/中断的时候，先调用\_\_alltraps(这个是由stvec这个寄存器的内容（地址）指定的)，然后跳入trap_handler函数处理（`call trap_handler`）,当这个函数返回的时候，就进入了\_\_restore函数，因为在\_\_alltraps以及trap_handler中都是处在s模式下的，并且使用的是内核栈，所以当返回到__restore后，restore的用途主要是更换栈为用户栈，并且恢复之前存储在内核栈上的寄存器信息，并在最后执行`sret`返回到U特权模式，从而完成恢复，并转而执行用户代码（地址由恢复出的ra给出，之前进入异常处理的时候保存现场保存了ra： `sd x1, 1*8(sp)`）。

##### **第二种情景**：

是在最开始任务第一次被调度的时候，这里是一个小trick来复用restore的功能，我展开来说说：

```rust
for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
            task.task_status = TaskStatus::Ready;
        }
```

这里对于每个任务进行初始化，主要是设置了TaskContext里面的ra sp 以及s0-s11寄存器，ra初始化为： `ra: __restore as usize`,而sp这里最初是使用内核栈，返回也是对应任务内核栈的地址（已经push了一些寄存器），这个sp里面已经push的寄存器是通过`init_app_cx(i)`实现的，push了：

<u>x0-x31、sstatus寄存器来记录执行的特权级别、sepc用于记录trap应该返回的指令地址，初始化的时候设置为任务函数起始地址</u>

其中，sp是x2，初始化的时候专门设置了x2的值不是0，而是任务用户栈的地址：    `cx.set_sp(sp); // app's user stack pointer`,sp的值是通过调用函数：`USER_STACK[app_id].get_sp()`得到的

而sepc的值是设置为entry(=`get_base_i(app_id)`就是初始化为任务函数的起始地址)

这里保存的一切，都通过`KERNEL_STACK[app_id].push_context`保存到内核栈了：表现为一个核心的函数：

```rust
/// get app info with entry and sp and save `TrapContext` in kernel stack
pub fn init_app_cx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].push_context(TrapContext::app_init_context(
        get_base_i(app_id),
        USER_STACK[app_id].get_sp(),
    ))
}
```

返回值即是内核栈的地址

以及这个最后包装完成的函数：

```rust
/// Create a new task context with a trap return addr and a kernel stack pointer
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
```



###### 讲了这么多前置，我们可以看看怎么复用的restore：

当任务第一次调度的时候执行switch函数，将current_task_cx_ptr代表的指向旧任务的内核栈指针保存到它的taskcontext（` sd sp, 8(a0)`）,以及保存ra和s0-s11,然后从新任务（这个时候这个任务是第一次调度，它的taskcontext保存的是上面提到的初始值），很重要的一点就是它里面保存的ra是__restore函数，这是复用的关键，switch后面就会恢复ra s0-s11以及sp（内核栈）的值，当执行`ret`命令后，就会转入restore执行，而restore就会恢复sp栈位用户栈，并且恢复spec的值为任务函数的起始地址，这样执行`sret`的时候，就会返回到用户程序执行，并且栈也是用户栈，并且特权级别也是U

#### 2、L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

代码来自于__restore前面几行

```assembly
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```

特殊处理的是sstatus、sepc和sscratch，这几个寄存器的操作只能使用csr的特殊指令，简单介绍一下：

- `csrrw`：原子地读取并写入 CSR（Control Status Register，控制状态寄存器）。例如，`csrrw sp, sscratch, sp` 会将 `sscratch` 寄存器的值写入 `sp`，并将 `sp` 的原始值写入 `sscratch`。
- `csrr`：读取 CSR 的值。例如，`csrr t0, sstatus` 会将 `sstatus` 寄存器的值读取到 `t0`。
- `csrw`：写入 CSR 的值。例如，`csrw sstatus, t0` 会将 `t0` 的值写入 `sstatus` 寄存器。

再拿出这个表来说明：

| CSR 名  | 该 CSR 与 Trap 相关的功能                                    |
| ------- | ------------------------------------------------------------ |
| sstatus | `SPP` 等字段给出 Trap 发生之前 CPU 处在哪个特权级（S/U）等信息 |
| sepc    | 当 Trap 是一个异常的时候，记录 Trap 发生之前执行的最后一条指令的地址 |
| scause  | 描述 Trap 的原因                                             |
| stval   | 给出 Trap 附加信息                                           |
| stvec   | 控制 Trap 处理代码的入口地址                                 |

所以上面几行代码就是先从内核栈里面取出sstatus->t0、sepc->t1、sscratch(也就是用户栈指针)->t2,然后将这三个从内核栈读出的之前保存的值恢复到这对应的三个特殊寄存器，<u>sret的时候会根据sstatus返回特权级别，根据sepc返回到用户指令，而sscratch则是通过`csrrw sp, sscratch, sp`将用户栈的值放入sp，将内核栈sp的值存入sscratch进行保存</u>。

#### 3、L50-L56：为何跳过了 `x2` 和 `x4`？

```assembly
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
   LOAD_GP %n
   .set n, n+1
.endr
```

##### 对于x2

因为X2是栈指针寄存器sp，对于它的处理，不仅需要把它从内核栈换为用户栈（也就是把2*8(sp)的值取出来给到sp，同时它当前出栈完成后所代表的内核栈的值也需要保存下来，我们需要保存到sscratch特殊寄存器里面），为了同时满足这两点，我们不得不对sp（x2）单独处理，在sp完成所有出栈后，将它的值保存入sscratch并且把sscratch的值（是用户栈的值，因为之前通过`ld t2, 2*8(sp)`和`csrw sscratch, t2`）把它取出来放入了sscratch的，这样只需要一条特殊寄存器的指令实现sp和sscratch的交换，让sp改为用户栈，sscratch保存内核栈：`csrrw sp, sscratch, sp`

#### 4、L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

这一个问题前面已经详细阐释了，这里直接说答案：

```assembly
csrrw sp, sscratch, sp
```

sp的值变为之前保存在内核栈是的用户栈的地址值，sscratch保存经过__restore恢复的内核栈的地址值，当下次异常处理的时候，会利用

```assembly
__alltraps:
  csrrw sp, sscratch, sp
```

将栈切换，所以保存sscratch很有必要，后续trap的时候都是通过它来改变sp为内核栈

#### 5、`__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

当然是`sret`了

前面也是阐释过的：

sret指令执行后，会将sepc的值加载到pc，完成跳转如用户程序执行，并且会从Supevisor模式中的trap返回（根据的是我们恢复的sstatus寄存器的内容，详见下面这个手册的图片），这样就完成指令执行后进入用户态（因为我们的sstatus恢复的时候是恢复的用户态的状态内容）

![image-20240427095646340](D:/data_space/typora_photo_data/image-20240427095646340.png)

#### 6、L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

这里前面也阐释过，

```assembly
csrrw sp, sscratch, sp
```

这个是在__alltraps的第一条指令，是进入trap处理异常/中断，这里就是进入异常的时候把sp从之前的用户栈转为内核栈，而把之前的用户栈保存如sscratch寄存器里面，执行完成后，sp就是内核栈的指针，而sscratch就是保存的用户栈指针。

#### 7、从 U 态进入 S 态是哪一条指令发生的？

当然是在用户程序里面通过包装的syscall，而这个函数通过汇编指令调用的ecall进入的s态啦，不过还有一种可能是用户程序发生异常比如访问非法地址，或者指令方法（如使用了sret等指令）

syscall的情况就是trap到UserEnvCall异常处理：

```rust

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}
```

而用户如果是访问非法地址（如0x0）、非法指令（如sret）这种，就会触发trap到内核s特权级别的处理，这个时候进入s特权模式，并且转入__alltraps函数执行（因为我们设置了stvec寄存器）
