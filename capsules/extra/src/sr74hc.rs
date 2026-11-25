// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Driver for SR74HC Shift Register
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! let sr = components::shift_register::ShiftRegisterComponent::new(
//!     latch,
//!     clock,
//!     data,
//! )
//! .finalize(components::shift_register_component_static!(
//!     GpioHw
//! ));
//! ```

use kernel::hil::gpio::Configure;
use kernel::hil::{gpio::Output, shift_register::ShiftRegister};

/// Simplified SR74HC Driver type to be used with one pin type.
pub type Sr74HC<P> = Sr74HCGeneric<P, P, P>;

/// Controls pins might come from different pin implementations, as boards might use
/// GPIO extenders.
pub struct Sr74HCGeneric<PL: Configure + Output, PC: Configure + Output, PD: Configure + Output> {
    latch: PL,
    clock: PC,
    data: PD,
}

impl<P: Configure + Output> Sr74HC<P> {
    pub fn new(latch: P, clock: P, data: P) -> Self {
        latch.make_output();
        clock.make_output();
        data.make_output();

        clock.clear();
        latch.set();
        data.clear();
        Self { latch, clock, data }
    }
}

impl<P: Configure + Output> ShiftRegister for Sr74HC<P> {
    fn write(&self, value: usize) {
        self.latch.clear();
        for i in 0..8 {
            if (value & (1 << i)) != 0 {
                self.data.set();
            } else {
                self.data.clear();
            }

            self.clock.clear();
            self.clock.set();

            self.latch.clear();
            self.latch.set();
        }
    }

    const BITS: usize = 4;
}
