use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led;
use nrf52832::gpio::Pin;

/// Panic.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(_pi: &PanicInfo) -> ! {
    let led_kernel_pin = &nrf52832::gpio::GPIOPin::new(Pin::P0_22);
    let led = &mut led::LedLow::new(led_kernel_pin);
    debug::panic_blink_forever(&mut [led])
}
