//! Virtual ADC Capsule
//!
//! Support Single Sample for now.

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;
use crate::virtual_alarm::VirtualMuxAlarm;

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
                            kernel::debug!("notify client");
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
pub struct AdcDeviceFake<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> {
    mux: &'a MuxAdcFake<'a, A, B>,
    channel: A::Channel,
    operation: OptionalCell<Operation>,
    next: ListLink<'a, AdcDeviceFake<'a, A, B>>,
    client: OptionalCell<&'a dyn hil::adc::Client>,
    alarm: &'a B,
}

impl<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> AdcDeviceFake<'a, A, B> {
    pub const fn new(mux: &'a MuxAdcFake<'a, A, B>, channel: A::Channel, alarm: &'a B) -> AdcDeviceFake<'a, A, B> {
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

impl<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> ListNode<'a, AdcDeviceFake<'a, A, B>> for AdcDeviceFake<'a, A, B> {
    fn next(&'a self) -> &'a ListLink<'a, AdcDeviceFake<'a, A, B>> {
        &self.next
    }
}

impl<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> hil::adc::AdcChannel for AdcDeviceFake<'a, A, B> {
    fn sample(&self) -> Result<(), ErrorCode> {
        self.operation.set(Operation::OneSample);
        kernel::debug!("sampling...");
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

impl<'a, A: hil::adc::Adc, B: hil::time::Alarm<'a>> hil::time::AlarmClient for AdcDeviceFake<'a, A, B> {
    fn alarm(&self) {
        self.mux.do_next_op();
        kernel::debug!("done sampling!");
    }
}
