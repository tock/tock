// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use core::panic::PanicInfo;
use kernel::utilities::cells::OptionalCell;

// use psc3::gpio::GpioPin;
use psc3::scb::Scb;

use kernel::debug::{self, IoWrite};
// use kernel::hil::led::LedHigh;

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {
    scb: OptionalCell<&'static Scb<'static>>,
}

impl Writer {
    pub fn set_scb(&self, scb: &'static Scb) {
        self.scb.set(scb);
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.scb.map(|scb| scb.transmit_uart_sync(s.as_bytes()));
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        self.scb.map(|scb| scb.transmit_uart_sync(buf));
        buf.len()
    }
}

pub static mut WRITER: Writer = Writer {
    scb: OptionalCell::empty(),
};

#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    use core::ptr::addr_of_mut;
    let writer = &mut *addr_of_mut!(WRITER);

    // todo gpio
    // let led_kernel_pin = &GpioPin::new(psoc62xa::gpio::PsocPin::P13_7);
    // let led = &mut LedHigh::new(led_kernel_pin);

    debug::panic_print::<_, _, _>(
        writer,
        pi,
        &cortexm33::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    // To satisfy the ! return type constraints.
    loop {}
}
