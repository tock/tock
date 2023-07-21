// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Virtual ADC Capsule
//!
//! Support Single Sample for now.

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

/// ADC Mux
pub struct MuxAdc<'a, A: hil::adc::Adc<'a>> {
    adc: &'a A,
    devices: List<'a, AdcDevice<'a, A>>,
    inflight: OptionalCell<&'a AdcDevice<'a, A>>,
}

impl<'a, A: hil::adc::Adc<'a>> hil::adc::Client for MuxAdc<'a, A> {
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

impl<'a, A: hil::adc::Adc<'a>> MuxAdc<'a, A> {
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

/// Virtual ADC device
pub struct AdcDevice<'a, A: hil::adc::Adc<'a>> {
    mux: &'a MuxAdc<'a, A>,
    channel: A::Channel,
    operation: OptionalCell<Operation>,
    next: ListLink<'a, AdcDevice<'a, A>>,
    client: OptionalCell<&'a dyn hil::adc::Client>,
}

impl<'a, A: hil::adc::Adc<'a>> AdcDevice<'a, A> {
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

impl<'a, A: hil::adc::Adc<'a>> ListNode<'a, AdcDevice<'a, A>> for AdcDevice<'a, A> {
    fn next(&'a self) -> &'a ListLink<'a, AdcDevice<'a, A>> {
        &self.next
    }
}

impl<'a, A: hil::adc::Adc<'a>> hil::adc::AdcChannel<'a> for AdcDevice<'a, A> {
    fn sample(&self) -> Result<(), ErrorCode> {
        self.operation.set(Operation::OneSample);
        self.mux.do_next_op();
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
    fn set_client(&self, client: &'a dyn hil::adc::Client) {
        self.client.set(client);
    }
}
