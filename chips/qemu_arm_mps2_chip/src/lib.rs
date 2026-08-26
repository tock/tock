// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Chip support for the ARM MPS2 AN385/AN386 FPGA images under QEMU.

#![no_std]

pub mod chip;
pub mod interrupts;
pub mod led;
pub mod timer;
pub mod uart;

#[cfg(feature = "cortex-m3")]
pub mod vectors_m3;
#[cfg(feature = "cortex-m4")]
pub mod vectors_m4;

use kernel::platform::chip::InterruptService;

/// The MPS2 AN385/AN386 machine's fixed system clock, in Hz (`SYSCLK_FRQ`
/// in QEMU's `hw/arm/mps2.c`), which every CMSDK peripheral's PCLK is
/// driven from.
pub const SYSCLK_FRQ: u32 = 25_000_000;

/// Instantiates the peripherals this chip crate drives.
///
/// Only UART0 and Timer0 are wired up as the console and alarm backing;
/// UART1-4 and Timer1 exist on the real memory map but are unused here.
pub struct Mps2DefaultPeripherals<'a> {
    pub uart0: uart::Uart<'a>,
    pub timer0: timer::Timer<'a>,
    pub fpgaio: led::Fpgaio,
}

impl Mps2DefaultPeripherals<'_> {
    pub fn new() -> Self {
        Self {
            uart0: uart::Uart::new(uart::UART0_BASE),
            timer0: timer::Timer::new(timer::TIMER0_BASE),
            fpgaio: led::Fpgaio::new(led::FPGAIO_BASE),
        }
    }
}

impl Default for Mps2DefaultPeripherals<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl InterruptService for Mps2DefaultPeripherals<'_> {
    fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::UART0_RX | interrupts::UART0_TX => self.uart0.handle_interrupt(),
            interrupts::TIMER0 => self.timer0.handle_interrupt(),
            _ => return false,
        }
        true
    }
}
