// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use crate::chip_specs::Stm32wle5jcSpecs;
use stm32wle5xx::chip::Stm32wle5xxDefaultPeripherals;

pub struct Stm32wle5jcDefaultPeripherals<'a> {
    pub stm32wle: Stm32wle5xxDefaultPeripherals<'a, Stm32wle5jcSpecs>,
}

impl<'a> Stm32wle5jcDefaultPeripherals<'a> {
    pub unsafe fn new(
        clocks: &'a crate::clocks::Clocks<'a, Stm32wle5jcSpecs>,
        exti: &'a crate::exti::Exti<'a>,
        syscfg: &'a crate::syscfg::Syscfg,
    ) -> Self {
        Self {
            stm32wle: Stm32wle5xxDefaultPeripherals::new(clocks, exti, syscfg),
        }
    }
    // Necessary for setting up circular dependencies & registering deferred
    // calls
    pub fn init(&'static self) {
        self.stm32wle.setup_circular_deps();
    }
}
impl kernel::platform::chip::InterruptService for Stm32wle5jcDefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        #[allow(clippy::match_single_binding)]
        match interrupt {
            // put Stm32wle5jc specific interrupts here
            _ => self.stm32wle.service_interrupt(interrupt),
        }
    }
}
