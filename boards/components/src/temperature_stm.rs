//! Component for the built-in STM temperature sensor.

use core::mem::MaybeUninit;
use core_capsules::virtual_adc::AdcDevice;
use extra_capsules::temperature_stm::TemperatureSTM;
use kernel::component::Component;
use kernel::hil::adc;
use kernel::hil::adc::AdcChannel;

#[macro_export]
macro_rules! temperature_stm_adc_component_static {
    ($A:ty $(,)?) => {{
        let adc_device = components::adc_component_static!($A);
        let temperature_stm =
            kernel::static_buf!(extra_capsules::temperature_stm::TemperatureSTM<'static>);

        (adc_device, temperature_stm)
    };};
}

pub struct TemperatureSTMComponent<A: 'static + adc::Adc> {
    adc_mux: &'static core_capsules::virtual_adc::MuxAdc<'static, A>,
    adc_channel: A::Channel,
    slope: f32,
    v_25: f32,
}

impl<A: 'static + adc::Adc> TemperatureSTMComponent<A> {
    pub fn new(
        adc_mux: &'static core_capsules::virtual_adc::MuxAdc<'static, A>,
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

impl<A: 'static + adc::Adc> Component for TemperatureSTMComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<AdcDevice<'static, A>>,
        &'static mut MaybeUninit<TemperatureSTM<'static>>,
    );
    type Output = &'static TemperatureSTM<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let adc_device =
            crate::adc::AdcComponent::new(self.adc_mux, self.adc_channel).finalize(s.0);

        let temperature_stm =
            s.1.write(TemperatureSTM::new(adc_device, self.slope, self.v_25));

        adc_device.set_client(temperature_stm);

        temperature_stm
    }
}
