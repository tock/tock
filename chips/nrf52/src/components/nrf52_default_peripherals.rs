// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Component for nRF52 default peripherals.

use core::mem::MaybeUninit;
use kernel::component::Component;

use crate::peripherals::Nrf52DefaultPeripherals;

#[macro_export]
macro_rules! nrf52_default_peripherals_component_static {
    ($c:ident $(,)?) => {{
        let peripherals = kernel::static_buf!($c::peripherals::Nrf52DefaultPeripherals);

        peripherals
    };};
}

pub struct Nrf52DefaultPeripheralsComponent {}

impl Nrf52DefaultPeripheralsComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for Nrf52DefaultPeripheralsComponent {
    type StaticInput = &'static mut MaybeUninit<Nrf52DefaultPeripherals<'static>>;
    type Output = &'static Nrf52DefaultPeripherals<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let peripherals = s.write(Nrf52DefaultPeripherals::new());

        peripherals
    }
}
