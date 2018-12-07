use core::intrinsics;
use core::panic::PanicInfo;

/// Panic handler. Adapted from `panic-abort` crate
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(_info: &PanicInfo) -> ! {
    intrinsics::abort();
}
