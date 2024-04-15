#![no_std]
#![no_main]
mod lang_items;

use core::arch::global_asm;

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    loop {}
}
global_asm!(include_str!("entry.asm"));

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
