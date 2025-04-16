// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use earlgrey::chip_config::EarlGreyConfig;
use kernel::debug;
use kernel::debug::IoWrite;

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
    fn write(&mut self, buf: &[u8]) -> usize {
        // This creates a second instance of the UART peripheral, and should only be used
        // during panic.
        earlgrey::uart::Uart::new(
            earlgrey::uart::UART0_BASE,
            crate::ChipConfig::PERIPHERAL_FREQ,
        )
        .transmit_sync(buf);
        buf.len()
    }
}

#[cfg(not(test))]
use kernel::hil::gpio::Configure;
#[cfg(not(test))]
use kernel::hil::led;

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    use core::ptr::{addr_of, addr_of_mut};
    let first_led_pin = &mut earlgrey::gpio::GpioPin::new(
        earlgrey::gpio::GPIO_BASE,
        earlgrey::pinmux::PadConfig::Output(
            earlgrey::registers::top_earlgrey::MuxedPads::Ioa6,
            earlgrey::registers::top_earlgrey::PinmuxOutsel::GpioGpio7,
        ),
        earlgrey::gpio::pins::pin7,
    );
    first_led_pin.make_output();
    let first_led = &mut led::LedLow::new(first_led_pin);
    let writer = &mut *addr_of_mut!(WRITER);

    #[cfg(feature = "sim_verilator")]
    debug::panic(
        &mut [first_led],
        writer,
        pi,
        &|| {},
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    );

    #[cfg(not(feature = "sim_verilator"))]
    debug::panic(
        &mut [first_led],
        writer,
        pi,
        &rv32i::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    );
}

#[cfg(test)]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut WRITER;

    #[cfg(feature = "sim_verilator")]
    debug::panic_print(writer, pi, &|| {}, &PROCESSES, &CHIP, &PROCESS_PRINTER);
    #[cfg(not(feature = "sim_verilator"))]
    debug::panic_print(
        writer,
        pi,
        &rv32i::support::nop,
        &PROCESSES,
        &CHIP,
        &PROCESS_PRINTER,
    );

    let _ = writeln!(writer, "{}", pi);
    // Exit QEMU with a return code of 1
    crate::tests::semihost_command_exit_failure();
}
