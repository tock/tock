use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::gpio;
use kernel::hil::led;

use crate::CHIP;
use crate::PROCESSES;

struct Writer {}

static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) {
        // This creates a second instance of the UART peripheral, and should only be used
        // during panic.
        earlgrey::uart::Uart::new(
            earlgrey::uart::UART0_BASE,
            earlgrey::chip_config::CONFIG.peripheral_freq,
        )
        .transmit_sync(buf);
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    let first_led_pin = &mut earlgrey::gpio::GpioPin::new(
        earlgrey::gpio::GPIO0_BASE,
        earlgrey::gpio::PADCTRL_BASE,
        earlgrey::gpio::pins::pin7,
    );
    gpio::Pin::make_output(first_led_pin);
    let first_led = &mut led::LedLow::new(first_led_pin);

    let writer = &mut WRITER;

    debug::panic(
        &mut [first_led],
        writer,
        pi,
        &rv32i::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
