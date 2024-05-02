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

## Day10-Day16 2024/4/18-2024/4/24
生病住院了，只是把lab2看完了，lab3开了头

## Day17-Day19 2024/4/25-2024/4/27

完成lab3的全部，包括简答题，感觉学到了很多东西，和之前的实时操作系统的内容也联系上了不少。

[博客记录lab3](https://liamy.clovy.top/article/OS_Tutorial/lab3)

## Day20 2024/4/28

### 事件： ch4看了有一半

感觉好难，主要是这部分文档感觉写的有点抽象，代码给的支离破碎。

## Day21 2024/4/29

### 事件：ch4快看完了

这次反复的参考文档，阅读代码，大脑快不够用了，主要是很多东西之前没有连成片，花时间反复看，记忆才能把它们串联起来，memory area 、memoryset 分页的映射模式，内核的地址空间，跳转到用户程序的地址空间，切换过程，好多好多，不过慢慢看懂了真的挺有意思的。

## Day22 2024/4/30

### 事件：ch4的lab做了一半，快结束了

但是感觉速度慢了，的确感觉可以不是很仔细的看文档，直接做lab，主要是时间太紧张了，打算后续的lab就这样来提升速度。

## Day23 2024/5/1

### 事件：ch4完成

加油啊

### 事件：ch5完成

冲刺了，改变策略过后速度有一定提升,不过我想还有个原因是进程这部分本身也学过，而且比前面的内容要简单（我感觉）

## Day24 2024/5/2

### 事件：ch6启动

开始搞文件系统了

### 事件：ch6结束

感觉现在的方法变成先做题，先移植上一个lab的功能，然后遇到不懂的地方再回去查看文档，这样做速度的确提升了，但是感觉有些地方也的确有遗漏，做完了再来补，还有问答题没做呀。

主要是马上要考arm处理器架构的半期，数据库的期末还有嵌入式操作系统的期末，得花不少时间去复习呀。
