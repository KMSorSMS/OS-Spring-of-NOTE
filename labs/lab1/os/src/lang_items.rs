use core::panic::PanicInfo;

#[panic_handler]
fn liam_panic(_info: &PanicInfo) -> !{
    loop{}
}