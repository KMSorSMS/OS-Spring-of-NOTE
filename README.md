# Daily Schedule for OS Tutorial Spring of Code 2024

- [2024春夏OS训练营](https://liamy.clovy.top/category/2024%E6%98%A5%E5%A4%8FOS%E8%AE%AD%E7%BB%83%E8%90%A5)：所有阶段的blog笔记

---

## Day 0-4 2024/4/8-2024/4/12

### 事件0：报名！

因为学长的推荐，正准备自己开始做rcore lab的时候突然在rcore的官方repo里面看到news:

 <u>开源操作系统训练营报名！</u>

wow，看到里面的正是自己想要了解学习的内容，一下子打起了12分精神，感觉很切合自己所在的嵌入式方向，并且完美的满足自己想要在更深平台上学习的想法（之前是在stm32的机器上跑过简单的ucOSII 实时操作系统）。

### 事件1：rust，启动！

感觉自己花在学rust的时间挺长的，主要是想更深入的学习这个语言（正巧大二上学了编译原理），在rustlings上花了不少时间，不想一个个说语法了，只是记得smart_pointers的特性很有意思狠狠的理解了，当然还有所有权（第一次见到在编译阶段去强调这个概念的语言，之前写malloc实验的时候有想过能不能在写语言的时候把内存的管理考虑好），option之类的东西和c++真的很像，前面的智能指针也是c++那一套的东西（有种写cs144的感觉）。范型的使用我就类比之前学java的时候的用法了，让我记忆深刻的还有rust对于错误处理包装成一个enum，居然是个枚举，还有它的宏，也太多了吧（学c的时候确实体会过宏的强大）。

最后10个algorithms花了小半天写完，确实算是对之前的学习合起来应用了一下。

记录的笔记我就留在个人博客上了，因为用的notion写博客，试试推送很方便，所以习惯了：

[Rust基础积累---常更](https://liamy.clovy.top/article/OS_Tutorial/rust_learn)

## Day5 2024/4/13

### 事件：学习riscV（这个部分会迭代更新）

参考资料（放在reference dir）： [RISC-V手册](http://riscvbook.com/chinese/RISC-V-Reader-Chinese-v1.pdf)（训练营给的版本老了点）

### 事件：lab1半完成（读完了）

学了一点ANSI转义序列，还有bss清零的骚操作（直接用rust写而不是之前用汇编完成）

还有调用rustsbi接口实现打印（不过这里感觉还没看的很仔细），

今天参加成电杯足球赛去了，做的不多（最近也在忙着复习操作系统半期考试😥）

## Day6 2024/4/14

### 事件：看并发 ch8部分

因为想和我们学校半期考试复习结合，所以就跳着先看看这里的并发（chapter8）

peterson算法感觉tutorial讲的少了点手动，建议是看看南大jyy的OS课里面讲的

详细笔记我归在lab部分博客了：[lab1 blog](https://liamy.clovy.top/article/OS_Tutorial/lab1)

## Day7 2024/4/15

### 事件：lab1敲一遍

这个lab1没有需要写的部分，主要是把一些基本内容学习，跟着敲一遍做好笔记就好了。

[lab1 blog](https://liamy.clovy.top/article/OS_Tutorial/lab1)

## Day8 2024/4/16

### 事件：lab1 finish

看了lab2 的前两个小节，重新去认识了链接脚本里面的细节（Entry的作用、如何保证的_start在第一个位置等）收藏一个回答：[Entry的作用](https://stackoverflow.com/questions/40606700/what-does-entry-mean-in-a-linker-script)



## Day9 2024/4/17

有个[incorrect usage of slice::from_raw_parts](https://doc.rust-lang.org/beta/core/slice/fn.from_raw_parts.html#incorrect-usage) 
