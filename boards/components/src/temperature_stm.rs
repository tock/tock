use capsules::temperature_stm::TemperatureSTM;
use capsules::virtual_adc::AdcDevice;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::adc;
use kernel::hil::adc::AdcChannel;
use kernel::static_init_half;

#[macro_export]
macro_rules! temperaturestm_adc_component_helper {
    ($A:ty, $channel: expr, $adc_mux: expr) => {{
        use capsules::temperature_stm::TemperatureSTM;
        use capsules::virtual_adc::AdcDevice;
        use core::mem::MaybeUninit;
        use kernel::hil::adc::Adc;
        let mut temperature_stm_adc: &'static capsules::virtual_adc::AdcDevice<'static, $A> =
            components::adc::AdcComponent::new($adc_mux, $channel)
                .finalize(components::adc_component_helper!($A));
        static mut temperature_stm: MaybeUninit<TemperatureSTM<'static>> = MaybeUninit::uninit();
        (&mut temperature_stm_adc, &mut temperature_stm)
    };};
}

pub struct TemperatureSTMComponent<A: 'static + adc::Adc> {
    _select: PhantomData<A>,
    slope: f32,
    v_25: f32,
}

impl<A: 'static + adc::Adc> TemperatureSTMComponent<A> {
    pub fn new(slope: f32, v_25: f32) -> TemperatureSTMComponent<A> {
        TemperatureSTMComponent {
            _select: PhantomData,
            slope: slope,
            v_25: v_25,
        }
    }
}

impl<A: 'static + adc::Adc> Component for TemperatureSTMComponent<A> {
    type StaticInput = (
        &'static AdcDevice<'static, A>,
        &'static mut MaybeUninit<TemperatureSTM<'static>>,
    );
    type Output = &'static TemperatureSTM<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let temperature_stm = static_init_half!(
            static_buffer.1,
            TemperatureSTM<'static>,
            TemperatureSTM::new(static_buffer.0, self.slope, self.v_25)
        );

        static_buffer.0.set_client(temperature_stm);

        temperature_stm
    }
}
