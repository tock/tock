// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! This configuration's `InterruptService`: `mps2_base`'s default
//! peripherals, plus UART1 for the second console.
//!
//! `mps2_base::Mps2DefaultPeripherals` only drives UART0, so it doesn't
//! route UART1's interrupts. This wraps it with a UART1 alongside, falling
//! through to the wrapped peripherals for everything else, and is what this
//! board's `main()` hands to `QemuArmMps2Chip` (via `alloc_chip`) instead of
//! `mps2_base::ChipHw`'s default.

use kernel::platform::chip::InterruptService;
use qemu_arm_mps2::{interrupts, uart};

/// This configuration's `InterruptService`, layering UART1 on top of the
/// peripherals `mps2_base` already builds.
pub struct Peripherals<'a> {
    base: &'a qemu_arm_mps2::Mps2DefaultPeripherals<'a>,
    pub uart1: &'a uart::Uart<'a>,
}

impl<'a> Peripherals<'a> {
    pub fn new(
        base: &'a qemu_arm_mps2::Mps2DefaultPeripherals<'a>,
        uart1: &'a uart::Uart<'a>,
    ) -> Self {
        Self { base, uart1 }
    }
}

impl<'a> InterruptService for Peripherals<'a> {
    fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::UART1_RX | interrupts::UART1_TX => {
                self.uart1.handle_interrupt();
                true
            }
            _ => self.base.service_interrupt(interrupt),
        }
    }
}
