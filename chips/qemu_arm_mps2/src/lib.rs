// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Peripheral drivers for the ARM MPS2 AN385/AN386 FPGA images under QEMU.
//!
//! Peripherals are common to both; each image's core and vector table are in
//! its own crate (`qemu_arm_mps2_an385`, `qemu_arm_mps2_an386`).

#![forbid(unsafe_code)]
#![no_std]

pub mod interrupts;
pub mod led;
pub mod spi;
pub mod timer;
pub mod uart;
pub mod watchdog;

use kernel::platform::chip::InterruptService;
use kernel::utilities::StaticRef;

/// The MPS2 AN385/AN386 machine's fixed system clock, in Hz (`SYSCLK_FRQ`
/// in QEMU's `hw/arm/mps2.c`), which every CMSDK peripheral's PCLK is
/// driven from.
pub const SYSCLK_FRQ: u32 = 25_000_000;

/// Instantiates the peripherals this chip crate drives.
pub struct Mps2DefaultPeripherals<'a> {
    pub uart0: uart::Uart<'a>,
    pub timer0: timer::Timer<'a>,
    pub fpgaio: led::Fpgaio,
    pub spi_shield0: spi::Spi<'a>,
    pub watchdog: watchdog::Watchdog,
}

impl Mps2DefaultPeripherals<'_> {
    pub fn new(
        uart0: StaticRef<uart::UartRegisters>,
        timer0: StaticRef<timer::TimerRegisters>,
        fpgaio: StaticRef<led::FpgaioRegisters>,
        spi_shield0: StaticRef<spi::SpiRegisters>,
        watchdog: StaticRef<watchdog::WatchdogRegisters>,
    ) -> Self {
        Self {
            uart0: uart::Uart::new(uart0),
            timer0: timer::Timer::new(timer0),
            fpgaio: led::Fpgaio::new(fpgaio),
            spi_shield0: spi::Spi::new(spi_shield0),
            watchdog: watchdog::Watchdog::new(watchdog),
        }
    }
}

impl InterruptService for Mps2DefaultPeripherals<'_> {
    fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::UART0_RX | interrupts::UART0_TX => self.uart0.handle_interrupt(),
            interrupts::TIMER0 => self.timer0.handle_interrupt(),
            interrupts::SPI_SHIELD => self.spi_shield0.handle_interrupt(),
            _ => return false,
        }
        true
    }
}
