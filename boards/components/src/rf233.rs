// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for communicating with an RF233 chip (802.15.4) connected via SPI.
//!
//! This provides one Component, RF233Component, which provides basic
//! packet-level interfaces for communicating with 802.15.4.
//!
//! Usage
//! -----
//! ```rust
//! let rf233 = components::rf233::RF233Component::new(
//!     rf233_spi,
//!     &peripherals.pa[09], // reset
//!     &peripherals.pa[10], // sleep
//!     &peripherals.pa[08], // irq
//!     &peripherals.pa[08],
//!     RADIO_CHANNEL,
//! )
//! .finalize(components::rf233_component_static!(sam4l::spi::SpiHw));
//! ```

use capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice;
use capsules_extra::rf233::RF233;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::radio::RadioConfig;
use kernel::hil::spi::{SpiMaster, SpiMasterDevice};
use kernel::hil::{self, radio};

// Setup static space for the objects.
#[macro_export]
macro_rules! rf233_component_static {
    ($S:ty $(,)?) => {{
        let spi_device = kernel::static_buf!(
            capsules_extra::rf233::RF233<
                'static,
                capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<'static, $S>,
            >
        );
        // The RF233 radio stack requires four buffers for its SPI operations:
        //
        //   1. buf: a packet-sized buffer for SPI operations, which is
        //      used as the read buffer when it writes a packet passed to it and the write
        //      buffer when it reads a packet into a buffer passed to it.
        //   2. rx_buf: buffer to receive packets into
        //   3 + 4: two small buffers for performing registers
        //      operations (one read, one write).
        let rf233_buf = kernel::static_buf!([u8; kernel::hil::radio::MAX_BUF_SIZE]);
        let rf233_reg_write =
            kernel::static_buf!([u8; capsules_extra::rf233::SPI_REGISTER_TRANSACTION_LENGTH]);
        let rf233_reg_read =
            kernel::static_buf!([u8; capsules_extra::rf233::SPI_REGISTER_TRANSACTION_LENGTH]);

        (spi_device, rf233_buf, rf233_reg_write, rf233_reg_read)
    };};
}

pub struct RF233Component<S: SpiMaster<'static> + 'static> {
    spi: &'static VirtualSpiMasterDevice<'static, S>,
    reset: &'static dyn hil::gpio::Pin,
    sleep: &'static dyn hil::gpio::Pin,
    irq: &'static dyn hil::gpio::InterruptPin<'static>,
    ctl: &'static dyn hil::gpio::InterruptPin<'static>,
    channel: radio::RadioChannel,
}

impl<S: SpiMaster<'static> + 'static> RF233Component<S> {
    pub fn new(
        spi: &'static VirtualSpiMasterDevice<'static, S>,
        reset: &'static dyn hil::gpio::Pin,
        sleep: &'static dyn hil::gpio::Pin,
        irq: &'static dyn hil::gpio::InterruptPin<'static>,
        ctl: &'static dyn hil::gpio::InterruptPin<'static>,
        channel: radio::RadioChannel,
    ) -> Self {
        Self {
            spi,
            reset,
            sleep,
            irq,
            ctl,
            channel,
        }
    }
}

impl<S: SpiMaster<'static> + 'static> Component for RF233Component<S> {
    type StaticInput = (
        &'static mut MaybeUninit<RF233<'static, VirtualSpiMasterDevice<'static, S>>>,
        &'static mut MaybeUninit<[u8; hil::radio::MAX_BUF_SIZE]>,
        &'static mut MaybeUninit<[u8; capsules_extra::rf233::SPI_REGISTER_TRANSACTION_LENGTH]>,
        &'static mut MaybeUninit<[u8; capsules_extra::rf233::SPI_REGISTER_TRANSACTION_LENGTH]>,
    );
    type Output = &'static RF233<'static, VirtualSpiMasterDevice<'static, S>>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let rf233_buf = s.1.write([0; hil::radio::MAX_BUF_SIZE]);
        let rf233_reg_write =
            s.2.write([0; capsules_extra::rf233::SPI_REGISTER_TRANSACTION_LENGTH]);
        let rf233_reg_read =
            s.3.write([0; capsules_extra::rf233::SPI_REGISTER_TRANSACTION_LENGTH]);
        let rf233 = s.0.write(RF233::new(
            self.spi,
            rf233_buf,
            rf233_reg_write,
            rf233_reg_read,
            self.reset,
            self.sleep,
            self.irq,
            self.channel,
        ));
        self.ctl.set_client(rf233);
        self.spi.set_client(rf233);
        let _ = rf233.initialize();
        rf233
    }
}
