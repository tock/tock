// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Component for nRF52840 default peripherals.

use core::mem::MaybeUninit;
use kernel::component::Component;

use crate::interrupt_service::Nrf52840DefaultPeripherals;

#[macro_export]
macro_rules! nrf52840_default_peripherals_component_static {
    ($(,)?) => {{
        let nrf52_static = nrf52840::nrf52_default_peripherals_component_static!(nrf52840);

        let peripherals =
            kernel::static_buf!(nrf52840::interrupt_service::Nrf52840DefaultPeripherals);
        let ieee802154_radio_ack_buf =
            kernel::static_buf!([u8; nrf52840::ieee802154_radio::ACK_BUF_SIZE]);

        (nrf52_static, peripherals, ieee802154_radio_ack_buf)
    };};
}

pub struct Nrf52840DefaultPeripheralsComponent {}

impl Nrf52840DefaultPeripheralsComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for Nrf52840DefaultPeripheralsComponent {
    type StaticInput = (
        &'static mut MaybeUninit<nrf52::peripherals::Nrf52DefaultPeripherals<'static>>,
        &'static mut MaybeUninit<Nrf52840DefaultPeripherals<'static>>,
        &'static mut MaybeUninit<[u8; crate::ieee802154_radio::ACK_BUF_SIZE]>,
    );
    type Output = &'static Nrf52840DefaultPeripherals<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let nrf52_peripherals =
            nrf52::components::nrf52_default_peripherals::Nrf52DefaultPeripheralsComponent::new()
                .finalize(s.0);

        let ack_buf = s.2.write([0; crate::ieee802154_radio::ACK_BUF_SIZE]);
        let peripherals = unsafe {
            s.1.write(Nrf52840DefaultPeripherals::new(nrf52_peripherals, ack_buf))
        };

        peripherals
    }
}
