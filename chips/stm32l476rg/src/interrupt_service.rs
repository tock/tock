// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

use crate::chip_specs::Stm32l476Specs;
use stm32l4xx::chip::Stm32l4xxDefaultPeripherals;

pub struct Stm32l476rgDefaultPeripherals<'a> {
    pub stm32l4: Stm32l4xxDefaultPeripherals<'a, Stm32l476Specs>,
    // Once implemented, place Stm32l476rg specific peripherals here
}

impl<'a> Stm32l476rgDefaultPeripherals<'a> {
    pub unsafe fn new(
        clocks: &'a crate::clocks::Clocks<'a, Stm32l476Specs>,
        exti: &'a crate::exti::Exti<'a>,
    ) -> Self {
        Self {
            stm32l4: Stm32l4xxDefaultPeripherals::new(clocks, exti),
        }
    }
    // Necessary for setting up circular dependencies & registering deferred
    // calls
    pub fn init(&'static self) {
        self.stm32l4.setup_circular_deps();
    }
}
impl kernel::platform::chip::InterruptService for Stm32l476rgDefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        #[allow(clippy::match_single_binding)]
        match interrupt {
            // put Stm32l476rg specific interrupts here
            _ => self.stm32l4.service_interrupt(interrupt),
        }
    }
}
