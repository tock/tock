use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::led;
use msp432::gpio::PinNr;

#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(_pi: &PanicInfo) -> ! {
    const LED1_PIN: PinNr = PinNr::P01_0;
    let led = &mut led::LedLow::new(&mut msp432::gpio::PINS[LED1_PIN as usize]);
    debug::panic_blink_forever(&mut [led]);
}
