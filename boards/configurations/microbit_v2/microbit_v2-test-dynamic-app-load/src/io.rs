// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;

use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart;
use nrf52833::gpio::Pin;
use nrf52833::uart::{Uarte, UARTE0_BASE};

use crate::CHIP;
use crate::PROCESSES;
use crate::PROCESS_PRINTER;

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
    fn write(&mut self, buf: &[u8]) -> usize {
        let uart = Uarte::new(UARTE0_BASE);

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
        buf.len()
    }
}

/// Default panic handler for the microbit board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // MicroBit v2 has an LED matrix, use the upper left LED
    // let mut led = Led (&gpio::PORT[Pin::P0_28], );

    // MicroBit v2 has a microphone LED, use it for panic

    use core::ptr::{addr_of, addr_of_mut};
    let led_kernel_pin = &nrf52833::gpio::GPIOPin::new(Pin::P0_20);
    let led = &mut led::LedLow::new(led_kernel_pin);
    let writer = &mut *addr_of_mut!(WRITER);
    debug::panic(
        &mut [led],
        writer,
        pi,
        &cortexm4::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    )
}
