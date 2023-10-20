// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for the MX25R6435F flash chip.
//!
//! Usage
//! -----
//! ```rust
//! let mx25r6435f = components::mx25r6435f::Mx25r6435fComponent::new(
//!     &gpio_port[driver.write_protect_pin],
//!     &gpio_port[driver.hold_pin],
//!     &gpio_port[driver.chip_select] as &dyn kernel::hil::gpio::Pin,
//!     mux_alarm,
//!     mux_spi,
//! )
//! .finalize(components::mx25r6435f_component_static!(
//!     nrf52::spi::SPIM,
//!     nrf52::gpio::GPIOPin,
//!     nrf52::rtc::Rtc
//! ));
//! ```

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules_core::virtualizers::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use capsules_extra::mx25r6435f::MX25R6435F;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::spi::SpiMasterDevice;
use kernel::hil::time::Alarm;

// Setup static space for the objects.
#[macro_export]
macro_rules! mx25r6435f_component_static {
    ($S:ty, $P:ty, $A:ty $(,)?) => {{
        let spi_device = kernel::static_buf!(
            capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<'static, $S>
        );
        let alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );
        let mx25r6435f = kernel::static_buf!(
            capsules_extra::mx25r6435f::MX25R6435F<
                'static,
                capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<'static, $S>,
                $P,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );

        let tx_buf = kernel::static_buf!([u8; capsules_extra::mx25r6435f::TX_BUF_LEN]);
        let rx_buf = kernel::static_buf!([u8; capsules_extra::mx25r6435f::RX_BUF_LEN]);

        (spi_device, alarm, mx25r6435f, tx_buf, rx_buf)
    };};
}

pub type Mx25r6435fComponentType<S, P, A> = capsules_extra::mx25r6435f::MX25R6435F<
    'static,
    capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<'static, S>,
    P,
    VirtualMuxAlarm<'static, A>,
>;

pub struct Mx25r6435fComponent<
    S: 'static + hil::spi::SpiMaster<'static>,
    P: 'static + hil::gpio::Pin,
    A: 'static + hil::time::Alarm<'static>,
> {
    write_protect_pin: Option<&'static P>,
    hold_pin: Option<&'static P>,
    chip_select: S::ChipSelect,
    mux_alarm: &'static MuxAlarm<'static, A>,
    mux_spi: &'static MuxSpiMaster<'static, S>,
}

impl<
        S: 'static + hil::spi::SpiMaster<'static>,
        P: 'static + hil::gpio::Pin,
        A: 'static + hil::time::Alarm<'static>,
    > Mx25r6435fComponent<S, P, A>
{
    pub fn new(
        write_protect_pin: Option<&'static P>,
        hold_pin: Option<&'static P>,
        chip_select: S::ChipSelect,
        mux_alarm: &'static MuxAlarm<'static, A>,
        mux_spi: &'static MuxSpiMaster<'static, S>,
    ) -> Mx25r6435fComponent<S, P, A> {
        Mx25r6435fComponent {
            write_protect_pin,
            hold_pin,
            chip_select,
            mux_alarm,
            mux_spi,
        }
    }
}

impl<
        S: 'static + hil::spi::SpiMaster<'static>,
        P: 'static + hil::gpio::Pin,
        A: 'static + hil::time::Alarm<'static>,
    > Component for Mx25r6435fComponent<S, P, A>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualSpiMasterDevice<'static, S>>,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<
            MX25R6435F<'static, VirtualSpiMasterDevice<'static, S>, P, VirtualMuxAlarm<'static, A>>,
        >,
        &'static mut MaybeUninit<[u8; capsules_extra::mx25r6435f::TX_BUF_LEN]>,
        &'static mut MaybeUninit<[u8; capsules_extra::mx25r6435f::RX_BUF_LEN]>,
    );
    type Output = &'static MX25R6435F<
        'static,
        VirtualSpiMasterDevice<'static, S>,
        P,
        VirtualMuxAlarm<'static, A>,
    >;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let mx25r6435f_spi = static_buffer
            .0
            .write(VirtualSpiMasterDevice::new(self.mux_spi, self.chip_select));
        // Create an alarm for this chip.
        let mx25r6435f_virtual_alarm = static_buffer.1.write(VirtualMuxAlarm::new(self.mux_alarm));
        mx25r6435f_virtual_alarm.setup();

        let tx_buf = static_buffer
            .3
            .write([0; capsules_extra::mx25r6435f::TX_BUF_LEN]);
        let rx_buf = static_buffer
            .4
            .write([0; capsules_extra::mx25r6435f::RX_BUF_LEN]);

        let mx25r6435f = static_buffer
            .2
            .write(capsules_extra::mx25r6435f::MX25R6435F::new(
                mx25r6435f_spi,
                mx25r6435f_virtual_alarm,
                tx_buf,
                rx_buf,
                self.write_protect_pin,
                self.hold_pin,
            ));
        mx25r6435f_spi.setup();
        mx25r6435f_spi.set_client(mx25r6435f);
        mx25r6435f_virtual_alarm.set_alarm_client(mx25r6435f);
        mx25r6435f
    }
}
