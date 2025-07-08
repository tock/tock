use core::panic::PanicInfo;

#[panic_handler]
pub unsafe fn panic_fmt(_panic_info: &PanicInfo) -> ! {
    loop {}
}
