use core::fmt::Write;
use core::panic::PanicInfo;

use kernel::debug::{self, IoWrite};
use kernel::hil::led::LedHigh;
use rp2040::gpio::{RPGpio, RPGpioPin};

use crate::CHIP;
use crate::PROCESSES;

use cortex_m_semihosting::hprint;

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {}

impl Writer {}

/// Global static for debug writer
pub static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) {
        for &c in buf {
            hprint!("{}", c as char).unwrap_or_else(|_| {});
        }
    }
}

/// Default panic handler for the Raspberry Pi Pico board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // LED is conneted to GPIO 25
    let led_kernel_pin = &RPGpioPin::new(RPGpio::GPIO25);
    let led = &mut LedHigh::new(led_kernel_pin);
    let writer = &mut WRITER;
    debug::panic(
        &mut [led],
        writer,
        pi,
        &cortexm0p::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
