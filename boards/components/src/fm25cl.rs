// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for the FM25CL FRAM chip.
//!
//! Uses a SPI Interface.
//!
//! Usage
//! -----
//! ```rust
//! let fm25cl = components::fm25cl::Fm25clComponent::new(spi_mux, stm32f429zi::gpio::PinId::PE03)
//!     .finalize(components::fm25cl_component_static!(stm32f429zi::spi::Spi));
//! ```

use capsules_core::virtualizers::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use capsules_extra::fm25cl::FM25CL;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::spi;
use kernel::hil::spi::SpiMasterDevice;

#[macro_export]
macro_rules! fm25cl_component_static {
    ($S:ty $(,)?) => {{
        let txbuffer = kernel::static_buf!([u8; capsules_extra::fm25cl::BUF_LEN]);
        let rxbuffer = kernel::static_buf!([u8; capsules_extra::fm25cl::BUF_LEN]);

        let spi = kernel::static_buf!(
            capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<'static, $S>
        );
        let fm25cl = kernel::static_buf!(
            capsules_extra::fm25cl::FM25CL<
                'static,
                capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<'static, $S>,
            >
        );

        (spi, fm25cl, txbuffer, rxbuffer)
    };};
}

pub struct Fm25clComponent<S: 'static + spi::SpiMaster<'static>> {
    spi_mux: &'static MuxSpiMaster<'static, S>,
    chip_select: S::ChipSelect,
}

impl<S: 'static + spi::SpiMaster<'static>> Fm25clComponent<S> {
    pub fn new(
        spi_mux: &'static MuxSpiMaster<'static, S>,
        chip_select: S::ChipSelect,
    ) -> Fm25clComponent<S> {
        Fm25clComponent {
            spi_mux,
            chip_select,
        }
    }
}

impl<S: 'static + spi::SpiMaster<'static>> Component for Fm25clComponent<S> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualSpiMasterDevice<'static, S>>,
        &'static mut MaybeUninit<FM25CL<'static, VirtualSpiMasterDevice<'static, S>>>,
        &'static mut MaybeUninit<[u8; capsules_extra::fm25cl::BUF_LEN]>,
        &'static mut MaybeUninit<[u8; capsules_extra::fm25cl::BUF_LEN]>,
    );
    type Output = &'static FM25CL<'static, VirtualSpiMasterDevice<'static, S>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let spi_device = static_buffer
            .0
            .write(VirtualSpiMasterDevice::new(self.spi_mux, self.chip_select));
        spi_device.setup();

        let txbuffer = static_buffer.2.write([0; capsules_extra::fm25cl::BUF_LEN]);
        let rxbuffer = static_buffer.3.write([0; capsules_extra::fm25cl::BUF_LEN]);

        let fm25cl = static_buffer
            .1
            .write(FM25CL::new(spi_device, txbuffer, rxbuffer));
        spi_device.set_client(fm25cl);

        fm25cl
    }
}
