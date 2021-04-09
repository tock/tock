use core::fmt::Write;
use core::panic::PanicInfo;

use cortexm4;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart;
use nrf52833::gpio::{self, Pin};

use kernel::hil::gpio::{Configure, Input, Output};

use crate::CHIP;
use crate::PROCESSES;

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

        use kernel::hil::uart::Configure;

        if !self.initialized {
            self.initialized = true;
            let _ = uart.configure(uart::Parameters {
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

struct MatrixLed(
    &'static gpio::GPIOPin<'static>,
    &'static gpio::GPIOPin<'static>,
);

impl led::Led for MatrixLed {
    fn init(&self) {
        self.0.make_output();
        self.1.make_output();
        self.1.clear();
    }
    fn on(&self) {
        self.1.set();
    }
    fn off(&self) {
        self.1.clear();
    }
    fn toggle(&self) {
        self.1.toggle();
    }
    fn read(&self) -> bool {
        self.1.read()
    }
}

/// Default panic handler for the microbit board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // MicroBit v2 has an LED matrix, use the upper left LED
    // let mut led = Led (&gpio::PORT[Pin::P0_28], );

    // MicroBit v2 has a microphone LED, use it for panic
    let led_kernel_pin = &nrf52833::gpio::GPIOPin::new(Pin::P0_20);
    let led = &mut led::LedLow::new(led_kernel_pin);
    // MatrixLed(&gpio::PORT[Pin::P0_28], &gpio::PORT[Pin::P0_21]);
    let writer = &mut WRITER;
    debug::panic(
        &mut [led],
        writer,
        pi,
        &cortexm4::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
