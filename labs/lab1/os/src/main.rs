//! The main module and entrypoint

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::global_asm;
#[macro_use]
mod console;
mod lang_items;
mod sbi;

// const SYSCALL_EXIT: usize = 93;

// const SYSCALL_WRITE: usize = 64;

// pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
//     syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
// }

// struct Stdout;

// impl Write for Stdout {
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         sys_write(1, s.as_bytes());
//         Ok(())
//     }
// }

// #[macro_export]
// macro_rules! print {
//     ($fmt: literal $(, $($arg: tt)+)?) => {
//         $crate::console::print(format_args!($fmt $(, $($arg)+)?));
//     }
// }

// #[macro_export]
// macro_rules! println {
//     ($fmt: literal $(, $($arg: tt)+)?) => {
//         print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
//     }
// }

// pub fn print(args: fmt::Arguments) {
//     Stdout.write_fmt(args).unwrap();
// }

// fn syscall(id: usize, args: [usize; 3]) -> isize {
//     let mut ret;
//     unsafe {
//         core::arch::asm!(
//             "ecall",
//             inlateout("x10") args[0] => ret,
//             in("x11") args[1],
//             in("x12") args[2],
//             in("x17") id,
//         );
//     }
//     ret
// }

// pub fn sys_exit(xstate: i32) -> isize {
//     syscall(SYSCALL_EXIT, [xstate as usize, 0, 0])
// }

/// the rust entry-point of os
#[no_mangle]
pub fn rust_main() {
    // unsafe {
    //     core::arch::asm!(
    //         "li a0, 100", // Load immediate
    //         "add a1, a0, a0", // Addition
    //         "sub a2, a0, a1", // Subtraction
    //         "mul a3, a0, a1", // Multiplication
    //         "div a4, a0, a1", // Division
    //         out("a0") _,
    //         out("a1") _,
    //         out("a2") _,
    //         out("a3") _,
    //         out("a4") _,
    //     );
    // }
    clear_bss();
    println!("hello");
    sbi::shutdown();
}
global_asm!(include_str!("entry.asm"));
/// clear BSS segment
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

// fn main() {
//     // println!("Hello, world!");
// }
