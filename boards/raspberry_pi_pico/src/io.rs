use core::fmt::Write;
use core::panic::PanicInfo;

use cortex_m_semihosting::hprintln;

/// Default panic handler for the Raspberry Pi Pico board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    loop {
        hprintln!("{}", pi);
    }
}
