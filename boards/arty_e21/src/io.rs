use arty_e21_chip;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::gpio;
use kernel::hil::led;
use rv32i;
use sifive;

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
        sifive::uart::Uart::new(arty_e21_chip::uart::UART0_BASE, 32_000_000).transmit_sync(buf);
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // turn off the non panic leds, just in case
    let led_green = &sifive::gpio::GpioPin::new(
        arty_e21_chip::gpio::GPIO0_BASE,
        sifive::gpio::pins::pin1,
        sifive::gpio::pins::pin1::SET,
        sifive::gpio::pins::pin1::CLEAR,
    );
    gpio::Pin::make_output(led_green);
    gpio::Pin::clear(led_green);

    let led_blue = &sifive::gpio::GpioPin::new(
        arty_e21_chip::gpio::GPIO0_BASE,
        sifive::gpio::pins::pin0,
        sifive::gpio::pins::pin0::SET,
        sifive::gpio::pins::pin0::CLEAR,
    );
    gpio::Pin::make_output(led_blue);
    gpio::Pin::clear(led_blue);

    let led_red_pin = &mut sifive::gpio::GpioPin::new(
        arty_e21_chip::gpio::GPIO0_BASE,
        sifive::gpio::pins::pin2,
        sifive::gpio::pins::pin2::SET,
        sifive::gpio::pins::pin2::CLEAR,
    );

    let led_red = &mut led::LedHigh::new(led_red_pin);
    let writer = &mut WRITER;
    debug::panic(
        &mut [led_red],
        writer,
        pi,
        &rv32i::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
