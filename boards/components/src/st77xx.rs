// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for the ST77XX screen.
//!
//! Usage
//! -----
//! ```rust
//!
//! let bus = components::bus::SpiMasterBusComponent::new().finalize(
//!     components::spi_bus_component_static!(
//!         // spi type
//!         nrf52840::spi::SPIM,
//!         // chip select
//!         &nrf52840::gpio::PORT[GPIO_D4],
//!         // spi mux
//!         spi_mux
//!     ),
//! );
//!
//! let tft = components::st77xx::ST77XXComponent::new(mux_alarm,
//!                                                    bus,
//!                                                    Some(&nrf52840::gpio::PORT[GPIO_D3]),
//!                                                    Some(&nrf52840::gpio::PORT[GPIO_D2]),
//!                                                    &capsules_extra::st77xx::ST7735).finalize(
//!     components::st77xx_component_static!(
//!         // bus type
//!         capsules_extra::bus::SpiMasterBus<
//!             'static,
//!             VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>,
//!         >,
//!         // timer type
//!         nrf52840::rtc::Rtc,
//!         // pin type
//!         nrf52::gpio::GPIOPin<'static>,
//!     ),
//! );
//! ```

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules_extra::bus;
use capsules_extra::st77xx::{ST77XXScreen, ST77XX};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::gpio;
use kernel::hil::time::{self, Alarm};

// Setup static space for the objects.
#[macro_export]
macro_rules! st77xx_component_static {
    ($B: ty, $A:ty, $P:ty $(,)?) => {{
        let buffer = kernel::static_buf!([u8; capsules_extra::st77xx::BUFFER_SIZE]);
        let sequence_buffer = kernel::static_buf!(
            [capsules_extra::st77xx::SendCommand; capsules_extra::st77xx::SEQUENCE_BUFFER_SIZE]
        );
        let st77xx_alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );
        let st77xx = kernel::static_buf!(
            capsules_extra::st77xx::ST77XX<
                'static,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
                $B,
                $P,
            >
        );

        (st77xx_alarm, st77xx, buffer, sequence_buffer)
    };};
}

pub struct ST77XXComponent<
    A: 'static + time::Alarm<'static>,
    B: 'static + bus::Bus<'static>,
    P: 'static + gpio::Pin,
> {
    alarm_mux: &'static MuxAlarm<'static, A>,
    bus: &'static B,
    dc: Option<&'static P>,
    reset: Option<&'static P>,
    screen: &'static ST77XXScreen,
}

impl<A: 'static + time::Alarm<'static>, B: 'static + bus::Bus<'static>, P: 'static + gpio::Pin>
    ST77XXComponent<A, B, P>
{
    pub fn new(
        alarm_mux: &'static MuxAlarm<'static, A>,
        bus: &'static B,
        dc: Option<&'static P>,
        reset: Option<&'static P>,
        screen: &'static ST77XXScreen,
    ) -> ST77XXComponent<A, B, P> {
        ST77XXComponent {
            alarm_mux,
            bus,
            dc,
            reset,
            screen,
        }
    }
}

impl<A: 'static + time::Alarm<'static>, B: 'static + bus::Bus<'static>, P: 'static + gpio::Pin>
    Component for ST77XXComponent<A, B, P>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<ST77XX<'static, VirtualMuxAlarm<'static, A>, B, P>>,
        &'static mut MaybeUninit<[u8; capsules_extra::st77xx::BUFFER_SIZE]>,
        &'static mut MaybeUninit<
            [capsules_extra::st77xx::SendCommand; capsules_extra::st77xx::SEQUENCE_BUFFER_SIZE],
        >,
    );
    type Output = &'static ST77XX<'static, VirtualMuxAlarm<'static, A>, B, P>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let st77xx_alarm = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        st77xx_alarm.setup();

        let buffer = static_buffer
            .2
            .write([0; capsules_extra::st77xx::BUFFER_SIZE]);
        let sequence_buffer = static_buffer.3.write(
            [capsules_extra::st77xx::SendCommand::Nop;
                capsules_extra::st77xx::SEQUENCE_BUFFER_SIZE],
        );

        let st77xx = static_buffer.1.write(ST77XX::new(
            self.bus,
            st77xx_alarm,
            self.dc,
            self.reset,
            buffer,
            sequence_buffer,
            self.screen,
        ));
        self.bus.set_client(st77xx);
        st77xx_alarm.set_alarm_client(st77xx);

        st77xx
    }
}
