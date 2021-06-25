use capsules::temperature_rp2040::TemperatureRp2040;
use capsules::virtual_adc::AdcDevice;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::adc;
use kernel::hil::adc::AdcChannel;
use kernel::static_init_half;

#[macro_export]
macro_rules! temperaturerp2040_adc_component_helper {
    ($A:ty, $channel:expr, $adc_mux:expr $(,)?) => {{
        use capsules::temperature_rp2040::TemperatureRp2040;
        use capsules::virtual_adc::AdcDevice;
        use core::mem::MaybeUninit;
        use kernel::hil::adc::Adc;
        let mut temperature_adc: &'static capsules::virtual_adc::AdcDevice<'static, $A> =
            components::adc::AdcComponent::new($adc_mux, $channel)
                .finalize(components::adc_component_helper!($A));
        static mut temperature: MaybeUninit<TemperatureRp2040<'static>> = MaybeUninit::uninit();
        (&mut temperature_adc, &mut temperature)
    };};
}

pub struct TemperatureRp2040Component<A: 'static + adc::Adc> {
    _select: PhantomData<A>,
    slope: f32,
    v_27: f32,
}

impl<A: 'static + adc::Adc> TemperatureRp2040Component<A> {
    pub fn new(slope: f32, v_27: f32) -> TemperatureRp2040Component<A> {
        TemperatureRp2040Component {
            _select: PhantomData,
            slope: slope,
            v_27: v_27,
        }
    }
}

impl<A: 'static + adc::Adc> Component for TemperatureRp2040Component<A> {
    type StaticInput = (
        &'static AdcDevice<'static, A>,
        &'static mut MaybeUninit<TemperatureRp2040<'static>>,
    );
    type Output = &'static TemperatureRp2040<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let temperature_stm = static_init_half!(
            static_buffer.1,
            TemperatureRp2040<'static>,
            TemperatureRp2040::new(static_buffer.0, self.slope, self.v_27)
        );

        static_buffer.0.set_client(temperature_stm);

        temperature_stm
    }
}
