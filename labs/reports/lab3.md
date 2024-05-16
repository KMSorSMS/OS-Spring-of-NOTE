# LAB3---袁子为

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

[rcore-lab3](https://liamy.clovy.top/article/OS_Tutorial/lab5)

## 简答作业

## 1.

*stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。*

*实际情况是轮到 p1 执行吗？为什么？*

不会，因为p2.stride在p2执行后，会加上一个pass（10），但是250+10会溢出，导致优先级发生反转，p2明明应该是260的stride，但是因为u8，则变成4，则下次执行还是p2，事实上，哪怕这样，直到p2又增长，变成254，仍然是p2执行，然后又一次溢出，变成8，然后又执行很多次，到p2.stride为248，然而还溢出，变成2，然后又是从252，执行，溢出，变成6，然后又执行变成246，执行，溢出，变成0，然后又变成250，回到一开始的样子，那么，p1永远不会执行，

所以实际情况是，p1永远得不到执行！！！！

## 2.

*我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， **在不考虑溢出的情况下** , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。*

![image-20240513224715014](https://cdn.jsdelivr.net/gh/KMSorSMS/picGallery@master/img/202405132247193.png)

**上面字写错了一个，应该是不考虑溢出☝️**

*已知以上结论，**考虑溢出的情况下**，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 `partial_cmp` 函数，假设两个 Stride 永远不会相等。*

**

```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

*TIPS: 使用 8 bits 存储 stride, BigStride = 255, 则: `(125 < 255) == false`, `(129 < 255) == true`.*

补全：

```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let left_stride: u8 = self.0 as u8;
        let right_stride: u8 = self.0 as u8;
        //BigStride为255
        if left_stride > right_strde {
        //如果两者距离超过Bigstride/2，说明右边发生溢出，且这个溢出要纳入考虑，那么实际上是左边小
		        if left_stride - right_stride > BigStride/2{
				        return Some(Ordering::Less);
		        }else{
				        return Some(Ordering::Greater);
        }else{
			      if right_stride - left_stride > BigStride/2{
							   return Some(Ordering::Greater);
			      }else{
					      return Some(Ordering::Less);
			      }
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

主要是之前的结论，保证了正常不溢出的话，一定不会超过BigStride/2（我们从64位里面截断取出8位），如果发生，说明发生了溢出，则溢出的数字一定更大，但是如果两个数字都发生了溢出，那么他们的差就不会超过BigStride/2（相当于同时都减少一个相同的数，那么差值不变），所以这个时候不用考虑溢出，从而这个比较是永远正确的（选取Bigstride为8位的最大数字，而我们的stride本身可以取64位没问题的喔）