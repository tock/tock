// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for the built-in STM temperature sensor.

use capsules_core::virtualizers::virtual_adc::AdcDevice;
use capsules_extra::temperature_stm::TemperatureSTM;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::adc;
use kernel::hil::adc::AdcChannel;

#[macro_export]
macro_rules! temperature_stm_adc_component_static {
    ($A:ty $(,)?) => {{
        let adc_device = components::adc_component_static!($A);
        let temperature_stm = kernel::static_buf!(
            capsules_extra::temperature_stm::TemperatureSTM<
                'static,
                capsules_core::virtualizers::virtual_adc::AdcDevice<'static, $A>,
            >
        );

        (adc_device, temperature_stm)
    };};
}

pub type TemperatureSTMComponentType<A> =
    capsules_extra::temperature_stm::TemperatureSTM<'static, A>;

pub struct TemperatureSTMComponent<A: 'static + adc::Adc<'static>> {
    adc_mux: &'static capsules_core::virtualizers::virtual_adc::MuxAdc<'static, A>,
    adc_channel: A::Channel,
    slope: f32,
    v_25: f32,
}

impl<A: 'static + adc::Adc<'static>> TemperatureSTMComponent<A> {
    pub fn new(
        adc_mux: &'static capsules_core::virtualizers::virtual_adc::MuxAdc<'static, A>,
        adc_channel: A::Channel,
        slope: f32,
        v_25: f32,
    ) -> TemperatureSTMComponent<A> {
        TemperatureSTMComponent {
            adc_mux,
            adc_channel,
            slope: slope,
            v_25: v_25,
        }
    }
}

impl<A: 'static + adc::Adc<'static>> Component for TemperatureSTMComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<AdcDevice<'static, A>>,
        &'static mut MaybeUninit<TemperatureSTM<'static, AdcDevice<'static, A>>>,
    );
    type Output = &'static TemperatureSTM<'static, AdcDevice<'static, A>>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let adc_device =
            crate::adc::AdcComponent::new(self.adc_mux, self.adc_channel).finalize(s.0);

        let temperature_stm =
            s.1.write(TemperatureSTM::new(adc_device, self.slope, self.v_25));

        adc_device.set_client(temperature_stm);

        temperature_stm
    }
}
