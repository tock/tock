use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;

/// Panic.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(_pi: &PanicInfo) -> ! {
    const LED1_PIN: usize = 22;
    let led = &mut led::LedLow::new(&mut nrf52832::gpio::PORT[LED1_PIN]);
    debug::panic_blink_forever(&mut [led])
}
