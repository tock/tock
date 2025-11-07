// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use crate::spi_bus;
use crate::CYW4343x;
use crate::CYW4343xBus;
use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::{gpio, spi, time};
use time::Alarm;

// Setup static space for the objects.
#[macro_export]
macro_rules! cyw4343_component_static {
    ($P:ty, $A:ty, $B:ty $(,)?) => {{
        let alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );
        let driver = kernel::static_buf!(
            $crate::CYW4343x<
                'static,
                $P,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
                $B,
            >
        );
        let buffer = kernel::static_buf!([u8; 1600]);

        (alarm, driver, buffer)
    }};
}

pub struct CYW4343xComponent<
    P: 'static + gpio::Pin,
    A: 'static + time::Alarm<'static>,
    B: 'static + CYW4343xBus<'static>,
> {
    pwr: &'static P,
    alarm: &'static MuxAlarm<'static, A>,
    bus: &'static B,
    clm: &'static [u8],
}

impl<
        P: 'static + gpio::Pin,
        A: 'static + time::Alarm<'static>,
        B: 'static + CYW4343xBus<'static>,
    > CYW4343xComponent<P, A, B>
{
    pub fn new(
        pwr: &'static P,
        alarm: &'static MuxAlarm<'static, A>,
        bus: &'static B,
        clm: &'static [u8],
    ) -> Self {
        Self {
            pwr,
            alarm,
            bus,
            clm,
        }
    }
}

impl<
        P: 'static + gpio::Pin,
        A: 'static + time::Alarm<'static>,
        B: 'static + CYW4343xBus<'static>,
    > Component for CYW4343xComponent<P, A, B>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<CYW4343x<'static, P, VirtualMuxAlarm<'static, A>, B>>,
        &'static mut MaybeUninit<[u8; 1600]>,
    );
    type Output = &'static CYW4343x<'static, P, VirtualMuxAlarm<'static, A>, B>;

    fn finalize(self, static_memory: Self::StaticInput) -> Self::Output {
        let alarm = static_memory.0.write(VirtualMuxAlarm::new(self.alarm));
        let buffer = static_memory.2.write([0; 1600]);
        alarm.setup();
        let driver = static_memory
            .1
            .write(CYW4343x::new(alarm, self.bus, self.pwr, self.clm, buffer));
        alarm.set_alarm_client(driver);
        self.bus.set_client(driver);
        driver
    }
}

#[macro_export]
macro_rules! cyw4343x_spi_bus_component_static {
    ($S:ty, $A:ty) => {{
        let extra = kernel::static_buf!([u8; $crate::spi_bus::WORD_SIZE]);
        let buffer = kernel::static_buf!([u8; $crate::spi_bus::MAX_PACKET_SIZE]);
        let alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );
        let bus = kernel::static_buf!(
            $crate::spi_bus::CYW4343xSpiBus<
                'static,
                $S,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );

        (extra, buffer, alarm, bus)
    }};
}

pub struct CYW4343xSpiBusComponent<
    A: 'static + time::Alarm<'static>,
    S: 'static + spi::SpiMasterDevice<'static>,
> {
    alarm: &'static MuxAlarm<'static, A>,
    spi: &'static S,
    fw: &'static [u8],
    nv: &'static [u8],
}

impl<A: 'static + time::Alarm<'static>, S: 'static + spi::SpiMasterDevice<'static>>
    CYW4343xSpiBusComponent<A, S>
{
    pub fn new(
        alarm: &'static MuxAlarm<'static, A>,
        spi: &'static S,
        fw: &'static [u8],
        nv: &'static [u8],
    ) -> Self {
        Self { alarm, spi, fw, nv }
    }
}

impl<A: 'static + time::Alarm<'static>, S: 'static + spi::SpiMasterDevice<'static>> Component
    for CYW4343xSpiBusComponent<A, S>
{
    type StaticInput = (
        &'static mut MaybeUninit<[u8; spi_bus::WORD_SIZE]>,
        &'static mut MaybeUninit<[u8; spi_bus::MAX_PACKET_SIZE]>,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<spi_bus::CYW4343xSpiBus<'static, S, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static spi_bus::CYW4343xSpiBus<'static, S, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_memory: Self::StaticInput) -> Self::Output {
        let extra = static_memory.0.write([0; spi_bus::WORD_SIZE]);
        let buf = static_memory.1.write([0; spi_bus::MAX_PACKET_SIZE]);

        let alarm = static_memory.2.write(VirtualMuxAlarm::new(self.alarm));
        alarm.setup();

        let bus = static_memory.3.write(spi_bus::CYW4343xSpiBus::new(
            self.spi, alarm, extra, buf, self.fw, self.nv,
        ));

        alarm.set_alarm_client(bus);
        self.spi.set_client(bus);

        bus
    }
}
