//! Virtual ADC Capsule
//! Support Single Sample for now

use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil;
use kernel::ReturnCode;

/// ADC Mux
pub struct MuxAdc<'a, A: hil::adc::Adc> {
    adc: &'a A,
    devices: List<'a, AdcDevice<'a, A>>,
    inflight: OptionalCell<&'a AdcDevice<'a, A>>,
}

impl<'a, A: hil::adc::Adc> hil::adc::Client for MuxAdc<'a, A> {
    fn sample_ready(&self, sample: u16) {
        for node in self.devices.iter() {
            self.inflight.map(|inflight| {
                if node.channel == inflight.channel {
                    node.operation.map(|operation| match operation {
                        Operation::OneSample => {
                            node.operation.clear();
                            node.client.map(|client| client.sample_ready(sample));
                        }
                    });
                }
            });
        }
        self.inflight.clear();
        self.do_next_op();
    }
}

impl<'a, A: hil::adc::Adc> MuxAdc<'a, A> {
    pub const fn new(adc: &'a A) -> MuxAdc<'a, A> {
        MuxAdc {
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
                        self.adc.sample(&node.channel);
                        true
                    }
                });
                if started {
                    self.inflight.set(node);
                } else {
                    self.do_next_op();
                }
            });
        } else {
            self.inflight.map(|node| {
                node.operation.take().map(|operation| {
                    match operation {
                        Operation::OneSample => {
                            self.adc.sample(&node.channel);
                        }
                    }
                    self.do_next_op();
                });
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

/// Virtual ADC device
pub struct AdcDevice<'a, A: hil::adc::Adc> {
    mux: &'a MuxAdc<'a, A>,
    channel: A::Channel,
    operation: OptionalCell<Operation>,
    next: ListLink<'a, AdcDevice<'a, A>>,
    client: OptionalCell<&'a dyn hil::adc::Client>,
}

impl<'a, A: hil::adc::Adc> AdcDevice<'a, A> {
    pub const fn new(mux: &'a MuxAdc<'a, A>, channel: A::Channel) -> AdcDevice<'a, A> {
        let adc_user = AdcDevice {
            mux: mux,
            channel: channel,
            operation: OptionalCell::empty(),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        };
        adc_user
    }

    pub fn add_to_mux(&'a self) {
        self.mux.devices.push_head(self);
    }
}

impl<'a, A: hil::adc::Adc> ListNode<'a, AdcDevice<'a, A>> for AdcDevice<'a, A> {
    fn next(&'a self) -> &'a ListLink<'a, AdcDevice<'a, A>> {
        &self.next
    }
}

impl<A: hil::adc::Adc> hil::adc::AdcChannel for AdcDevice<'_, A> {
    fn sample(&self) -> ReturnCode {
        self.operation.set(Operation::OneSample);
        self.mux.do_next_op();
        ReturnCode::SUCCESS
    }

    fn stop_sampling(&self) -> ReturnCode {
        self.operation.clear();
        self.mux.do_next_op();
        ReturnCode::SUCCESS
    }

    fn sample_continuous(&self) -> ReturnCode {
        ReturnCode::ENOSUPPORT
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
