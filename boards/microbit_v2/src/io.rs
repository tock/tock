use core::fmt::Write;
use core::panic::PanicInfo;

use cortexm4;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart::{self};
use nrf52833::gpio::Pin;

use crate::CHIP;
use crate::PROCESSES;

use kernel::hil::uart::Configure;

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {
    initialized: bool,
}

impl Writer {
    /// Indicate that USART has already been initialized.
    pub fn set_initialized(&mut self) {
        self.initialized = true;
    }
}

/// Global static for debug writer
pub static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) {
        let uart = nrf52833::uart::Uarte::new();

        if !self.initialized {
            self.initialized = true;
            uart.configure(uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            });
        }

        unsafe {
            for &c in buf {
                uart.send_byte(c);
                while !uart.tx_ready() {}
            }
        }
    }
}

struct NoLed;

impl led::Led for NoLed {
    fn init(&mut self) {}
    fn on(&mut self) {}
    fn off(&mut self) {}
    fn toggle(&mut self) {}
    fn read(&self) -> bool { false }
}

/// Default panic handler for the microbit board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // MicroBit v2 has no LEDs
    let mut led = NoLed;
    let writer = &mut WRITER;
    debug::panic(
        &mut [&mut led],
        writer,
        pi,
        &cortexm4::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
