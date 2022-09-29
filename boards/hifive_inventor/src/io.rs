use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use e310_g003;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use rv32i;

use crate::CHIP;
use crate::PROCESSES;
use crate::PROCESS_PRINTER;

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
        let uart = sifive::uart::Uart::new(e310_g003::uart::UART0_BASE, 16_000_000);
        uart.transmit_sync(buf);
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    let led = sifive::gpio::GpioPin::new(
        e310_g003::gpio::GPIO0_BASE,
        sifive::gpio::pins::pin22,
        sifive::gpio::pins::pin22::SET,
        sifive::gpio::pins::pin22::CLEAR,
    );
    let led = &mut led::LedLow::new(&led);
    let writer = &mut WRITER;

    debug::panic(
        &mut [led],
        writer,
        pi,
        &rv32i::support::nop,
        &PROCESSES,
        &CHIP,
        &PROCESS_PRINTER,
    )
}
