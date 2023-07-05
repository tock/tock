// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for the Japan Display LPM013M126 display.
//!
//! Usage
//! -----
//!
//! ```rust
//! // Optional
//! let spi_device = static_init!(
//!     VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>,
//!     VirtualSpiMasterDevice::new(
//!         mux_spi,
//!         &nrf52840_peripherals.gpio_port[Pin::P0_05], // CS pin
//!     ),
//! );
//! let display
//!     = components::lpm013m126::Lpm013m126Component::new(
//!         disp_pin,
//!         extcomin_pin,
//!         alarm_mux,
//!     )
//!     .finalize(
//!         components::lpm013m126_component_static!(
//!             nrf52840::rtc::Rtc<'static>,
//!             nrf52840::gpio::GPIOPin,
//!             VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>,
//!             spi_mux,
//!             cs_pin,
//!         )
//!     );
//! display.initialize().unwrap();
//! // wait for `ScreenClient::screen_is_ready` callback
//! ```

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules_core::virtualizers::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use capsules_extra::lpm013m126::Lpm013m126;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::gpio;
use kernel::hil::spi::{SpiMaster, SpiMasterDevice};
use kernel::hil::time::Alarm;

/// CS is active high
pub struct Inverted<'a, P: gpio::Pin>(pub &'a P);

impl<'a, P: gpio::Pin> gpio::Configure for Inverted<'a, P> {
    fn configuration(&self) -> gpio::Configuration {
        self.0.configuration()
    }
    fn make_output(&self) -> gpio::Configuration {
        self.0.make_output()
    }
    fn disable_output(&self) -> gpio::Configuration {
        self.0.disable_output()
    }
    fn make_input(&self) -> gpio::Configuration {
        self.0.make_input()
    }
    fn disable_input(&self) -> gpio::Configuration {
        self.0.disable_input()
    }
    fn deactivate_to_low_power(&self) {
        self.0.deactivate_to_low_power()
    }
    fn set_floating_state(&self, _: gpio::FloatingState) {
        unimplemented!() // not sure what it looks like with inversion
    }
    fn floating_state(&self) -> gpio::FloatingState {
        unimplemented!() // not sure what it looks like with inversion
    }
}

impl<'a, P: gpio::Pin> gpio::Output for Inverted<'a, P> {
    fn set(&self) {
        self.0.clear()
    }
    fn clear(&self) {
        self.0.set()
    }
    fn toggle(&self) -> bool {
        self.0.toggle()
    }
}

impl<'a, P: gpio::Pin> gpio::Input for Inverted<'a, P> {
    fn read(&self) -> bool {
        !self.0.read()
    }
}

/// Setup static space for the driver and its requirements.
#[macro_export]
macro_rules! lpm013m126_component_static {
    ($A:ty, $P:ty, $S:ty $(,)?) => {{
        let alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );
        let buffer = kernel::static_buf!([u8; capsules_extra::lpm013m126::BUF_LEN]);
        let chip_select = kernel::static_buf!(components::lpm013m126::Inverted<'static, $P>);
        let spi_device = kernel::static_buf!(
            capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<'static, $S>
        );
        let lpm013m126 = kernel::static_buf!(
            capsules_extra::lpm013m126::Lpm013m126<
                'static,
                VirtualMuxAlarm<'static, $A>,
                $P,
                VirtualSpiMasterDevice<'static, $S>,
            >
        );

        (alarm, buffer, chip_select, spi_device, lpm013m126)
    }};
}

pub struct Lpm013m126Component<A, P, S>
where
    A: 'static + Alarm<'static>,
    P: 'static + gpio::Pin,
    P: gpio::Pin,
    S: 'static + SpiMaster<'static>,
{
    spi: &'static MuxSpiMaster<'static, S>,
    chip_select: &'static P,
    disp: &'static P,
    extcomin: &'static P,
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A, P, S> Lpm013m126Component<A, P, S>
where
    A: 'static + Alarm<'static>,
    P: 'static + gpio::Pin,
    P: gpio::Pin,
    S: 'static + SpiMaster<'static>,
{
    pub fn new(
        spi: &'static MuxSpiMaster<'static, S>,

        chip_select: &'static P,
        disp: &'static P,
        extcomin: &'static P,
        alarm_mux: &'static MuxAlarm<'static, A>,
    ) -> Self {
        Self {
            spi,
            chip_select,
            disp,
            extcomin,
            alarm_mux,
        }
    }
}

impl<A, P, S> Component for Lpm013m126Component<A, P, S>
where
    A: 'static + Alarm<'static>,
    P: 'static + gpio::Pin,
    S: 'static + SpiMaster<'static, ChipSelect = &'static mut Inverted<'static, P>>,
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<[u8; capsules_extra::lpm013m126::BUF_LEN]>,
        &'static mut MaybeUninit<Inverted<'static, P>>,
        &'static mut MaybeUninit<VirtualSpiMasterDevice<'static, S>>,
        &'static mut MaybeUninit<
            Lpm013m126<'static, VirtualMuxAlarm<'static, A>, P, VirtualSpiMasterDevice<'static, S>>,
        >,
    );
    type Output = &'static Lpm013m126<
        'static,
        VirtualMuxAlarm<'static, A>,
        P,
        VirtualSpiMasterDevice<'static, S>,
    >;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let lpm013m126_alarm = s.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        lpm013m126_alarm.setup();

        let buffer = s.1.write([0; capsules_extra::lpm013m126::BUF_LEN]);

        let chip_select = s.2.write(Inverted(self.chip_select));

        let spi_device =
            s.3.write(VirtualSpiMasterDevice::new(self.spi, chip_select));
        spi_device.setup();

        let lpm013m126 = s.4.write(
            Lpm013m126::new(
                spi_device,
                self.extcomin,
                self.disp,
                lpm013m126_alarm,
                buffer,
            )
            .unwrap(),
        );
        spi_device.set_client(lpm013m126);
        lpm013m126_alarm.set_alarm_client(lpm013m126);
        // Because this capsule uses multiple deferred calls internally, this
        // takes care of registering the deferred calls as well. Thus there is
        // no need to explicitly call
        // `kernel::deferred_call::DeferredCallClient::register`.
        lpm013m126.setup().unwrap();
        lpm013m126
    }
}
