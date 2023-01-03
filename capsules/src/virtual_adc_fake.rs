//! Fake Virtual ADC Capsule, for testing purposes
//!
//! Support Single Sample for now.
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let mux_alarm1 = components::alarm::AlarmMuxComponent::new(rtc)
//!    .finalize(components::alarm_mux_component_helper!(nrf52::rtc::Rtc));
//!
//! let virtual_alarm_adc = static_init!(
//!    capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>,
//!    capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm1)
//! );
//! virtual_alarm_adc.setup();
//!
//! let adc_mux = components::adc::AdcMuxFakeComponent::new(&base_peripherals.adc)
//!    .finalize(components::adc_mux_fake_component_helper!(nrf52833::adc::Adc, capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>));
//!
//! 
//! let adc_syscall =
//!    components::adc::AdcVirtualComponent::new(board_kernel, capsules::adc::DRIVER_NUM)
//!        .finalize(components::adc_syscall_fake_component_helper!(
//!            // ADC Ring 0 (P0)
//!            components::adc::AdcFakeComponent::new(
//!                &adc_mux,
//!                nrf52833::adc::AdcChannelSetup::new(nrf52833::adc::AdcChannel::AnalogInput0),
//!                &virtual_alarm_adc,
//!            )
//!            .finalize(components::adc_fake_component_helper!(nrf52833::adc::Adc, capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>,)),
//!            // ADC Ring 1 (P1)
//!            components::adc::AdcFakeComponent::new(
//!                &adc_mux,
//!                nrf52833::adc::AdcChannelSetup::new(nrf52833::adc::AdcChannel::AnalogInput1),
//!                &virtual_alarm_adc
//!            )
//!            .finalize(components::adc_fake_component_helper!(nrf52833::adc::Adc, capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>,)),
//!            // ADC Ring 2 (P2)
//!            components::adc::AdcFakeComponent::new(
//!                &adc_mux,
//!                nrf52833::adc::AdcChannelSetup::new(nrf52833::adc::AdcChannel::AnalogInput2),
//!                &virtual_alarm_adc
//!            )
//!            .finalize(components::adc_fake_component_helper!(nrf52833::adc::Adc, capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>,))
//!        ));
//! ```

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

/// ADC Mux
pub struct MuxAdcFake<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> {
    adc: &'a A,
    devices: List<'a, AdcDeviceFake<'a, A, B>>,
    inflight: OptionalCell<&'a AdcDeviceFake<'a, A, B>>,
}

impl<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> hil::adc::Client for MuxAdcFake<'a, A, B> {
    fn sample_ready(&self, sample: u16) {
        self.inflight.take().map(|inflight| {
            for node in self.devices.iter() {
                if node.channel == inflight.channel {
                    node.operation.take().map(|operation| match operation {
                        Operation::OneSample => {
                            node.client.map(|client| client.sample_ready(sample))
                        }
                    });
                }
            }
        });
        self.do_next_op();
    }
}

impl<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> MuxAdcFake<'a, A, B> {
    pub const fn new(adc: &'a A) -> MuxAdcFake<'a, A, B> {
        MuxAdcFake {
            adc: adc,
            devices: List::new(),
            inflight: OptionalCell::empty(),
        }
    }

    fn do_next_op(&self) {
        if self.inflight.is_none() {
            let mnode = self.devices.iter().find(|node| node.operation.is_some());
            mnode.map(|node| {
                let started = node.operation.map_or(false, |operation| match operation {
                    Operation::OneSample => {
                        let _ = self.adc.sample(&node.channel);
                        true
                    }
                });
                if started {
                    self.inflight.set(node);
                } else {
                    self.do_next_op();
                }
            });
        }
    }

    pub fn get_resolution_bits(&self) -> usize {
        self.adc.get_resolution_bits()
    }

    pub fn get_voltage_reference_mv(&self) -> Option<usize> {
        self.adc.get_voltage_reference_mv()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum Operation {
    OneSample,
}

/// Fake ADC device, for testing
/// Sample function sets an alarm
pub struct AdcDeviceFake<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> {
    mux: &'a MuxAdcFake<'a, A, B>,
    channel: A::Channel,
    operation: OptionalCell<Operation>,
    next: ListLink<'a, AdcDeviceFake<'a, A, B>>,
    client: OptionalCell<&'a dyn hil::adc::Client>,
    alarm: &'a B,
}

impl<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> AdcDeviceFake<'a, A, B> {
    pub const fn new(
        mux: &'a MuxAdcFake<'a, A, B>,
        channel: A::Channel,
        alarm: &'a B,
    ) -> AdcDeviceFake<'a, A, B> {
        let adc_user = AdcDeviceFake {
            mux: mux,
            channel: channel,
            operation: OptionalCell::empty(),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            alarm: alarm,
        };
        adc_user
    }

    pub fn add_to_mux(&'a self) {
        self.mux.devices.push_head(self);
    }
}

impl<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> ListNode<'a, AdcDeviceFake<'a, A, B>>
    for AdcDeviceFake<'a, A, B>
{
    fn next(&'a self) -> &'a ListLink<'a, AdcDeviceFake<'a, A, B>> {
        &self.next
    }
}

impl<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> hil::adc::AdcChannel
    for AdcDeviceFake<'a, A, B>
{
    fn sample(&self) -> Result<(), ErrorCode> {
        self.operation.set(Operation::OneSample);
        self.alarm.set_alarm(self.alarm.now(), B::Ticks::from(3000));
        Ok(())
    }

    fn stop_sampling(&self) -> Result<(), ErrorCode> {
        self.operation.clear();
        self.mux.do_next_op();
        Ok(())
    }

    fn sample_continuous(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn get_resolution_bits(&self) -> usize {
        self.mux.get_resolution_bits()
    }

    fn get_voltage_reference_mv(&self) -> Option<usize> {
        self.mux.get_voltage_reference_mv()
    }
    fn set_client(&self, client: &'static dyn hil::adc::Client) {
        self.client.set(client);
    }
}

impl<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> hil::time::AlarmClient
    for AdcDeviceFake<'a, A, B>
{
    fn alarm(&self) {
        self.mux.do_next_op();
    }
}
