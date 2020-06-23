use crate::CHIP;
use crate::PROCESSES;

use core::fmt::Write;
use core::panic::PanicInfo;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use msp432::gpio::PinNr;

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
        unsafe {
            msp432::uart::UART0.transmit_sync(buf);
        }
    }
}

#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(info: &PanicInfo) -> ! {
    const LED1_PIN: PinNr = PinNr::P01_0;
    let led = &mut led::LedHigh::new(&mut msp432::gpio::PINS[LED1_PIN as usize]);
    let writer = &mut UART;

    debug::panic(
        &mut [led],
        writer,
        info,
        &cortexm4::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
