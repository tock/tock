use crate::CHIP;
use crate::PROCESSES;

use core::fmt::Write;
use core::panic::PanicInfo;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use msp432::gpio::IntPinNr;
use msp432::wdt::Wdt;

/// Uart is used by kernel::debug to panic message to the serial port.
pub struct Uart {
    initialized: bool,
}

/// Global static for debug writer
pub static mut UART: Uart = Uart { initialized: false };

impl Uart {
    /// Indicate that UART has already been initialized.
    pub fn set_initialized(&mut self) {
        self.initialized = true;
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Uart {
    fn write(&mut self, buf: &[u8]) {
        let uart0 = msp432::uart::Uart::new(msp432::usci::USCI_A0_BASE, 0, 1, 1, 1);
        uart0.transmit_sync(buf);
    }
}

/// Panic handler
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(info: &PanicInfo) -> ! {
    const LED1_PIN: IntPinNr = IntPinNr::P01_0;
    let gpio_pin = msp432::gpio::IntPin::new(LED1_PIN);
    let led = &mut led::LedHigh::new(&gpio_pin);
    let writer = &mut UART;
    let wdt = Wdt::new();

    wdt.disable();
    debug::panic(
        &mut [led],
        writer,
        info,
        &cortexm4::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
