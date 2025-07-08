// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for the RaspberryPI 2040 built-in temperature sensor.

use capsules_core::virtualizers::virtual_adc::AdcDevice;
use capsules_extra::temperature_rp2040::TemperatureRp2040;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::adc;
use kernel::hil::adc::AdcChannel;

#[macro_export]
macro_rules! temperature_rp2040_adc_component_static {
    ($A:ty $(,)?) => {{
        let adc_device = components::adc_component_static!($A);
        let temperature_rp2040 = kernel::static_buf!(
            capsules_extra::temperature_rp2040::TemperatureRp2040<
                'static,
                capsules_core::virtualizers::virtual_adc::AdcDevice<'static, $A>,
            >
        );

        (adc_device, temperature_rp2040)
    };};
}

pub type TemperatureRp2040ComponentType<A> =
    capsules_extra::temperature_rp2040::TemperatureRp2040<'static, A>;

pub struct TemperatureRp2040Component<A: 'static + adc::Adc<'static>> {
    adc_mux: &'static capsules_core::virtualizers::virtual_adc::MuxAdc<'static, A>,
    adc_channel: A::Channel,
    slope: f32,
    v_27: f32,
}

impl<A: 'static + adc::Adc<'static>> TemperatureRp2040Component<A> {
    pub fn new(
        adc_mux: &'static capsules_core::virtualizers::virtual_adc::MuxAdc<'static, A>,
        adc_channel: A::Channel,
        slope: f32,
        v_27: f32,
    ) -> TemperatureRp2040Component<A> {
        TemperatureRp2040Component {
            adc_mux,
            adc_channel,
            slope,
            v_27,
        }
    }
}

impl<A: 'static + adc::Adc<'static>> Component for TemperatureRp2040Component<A> {
    type StaticInput = (
        &'static mut MaybeUninit<AdcDevice<'static, A>>,
        &'static mut MaybeUninit<TemperatureRp2040<'static, AdcDevice<'static, A>>>,
    );
    type Output = &'static TemperatureRp2040<'static, AdcDevice<'static, A>>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let adc_device =
            crate::adc::AdcComponent::new(self.adc_mux, self.adc_channel).finalize(s.0);

        let temperature_rp2040 =
            s.1.write(TemperatureRp2040::new(adc_device, self.slope, self.v_27));

        adc_device.set_client(temperature_rp2040);

        temperature_rp2040
    }
}
