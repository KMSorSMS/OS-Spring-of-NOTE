#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]


#[macro_use]
pub mod console;
mod lang_items;
mod syscall;
pub use console::{flush, STDIN, STDOUT};

extern crate alloc;
// extern crate core;

use syscall::*;


#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    // panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(
            start_bss as usize as *mut u8,
            end_bss as usize - start_bss as usize,
        )
        .fill(0);
    }
}

pub fn exit(exit_code: i32) -> ! {
    flush();
    sys_exit(exit_code);
}

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}