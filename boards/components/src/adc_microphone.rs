//! Component for ADC Microphone
//!
//! Usage
//! -----
//!
//!
//! ```rust
//! let adc_microphone = components::adc_microphone::AdcMicrophoneComponent::new().finalize(
//!     components::adc_microphone_component_helper!(
//!         // adc
//!         nrf52833::adc::Adc,
//!         // adc channel
//!             nrf52833::adc::AdcChannelSetup::setup(
//!             nrf52833::adc::AdcChannel::AnalogInput3,
//!             nrf52833::adc::AdcChannelGain::Gain4,
//!             nrf52833::adc::AdcChannelResistor::Bypass,
//!             nrf52833::adc::AdcChannelResistor::Pulldown,
//!             nrf52833::adc::AdcChannelSamplingTime::us3
//!         ),
//!         // adc mux
//!         adc_mux,
//!         // buffer size
//!         50,
//!         // gpio
//!         nrf52833::gpio::GPIOPin,
//!         // optional gpio pin
//!         Some(&base_peripherals.gpio_port[LED_MICROPHONE_PIN])
//!    ),
//! );
//! ```

use capsules::adc_microphone::AdcMicrophone;
use capsules::virtual_adc::AdcDevice;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::adc::{self, AdcChannel};
use kernel::hil::gpio;
use kernel::static_init_half;

#[macro_export]
macro_rules! adc_microphone_component_helper {
    ($A:ty, $channel:expr, $adc_mux:expr, $LEN:literal, $P: ty, $pin: expr $(,)?) => {{
        use capsules::adc_microphone::AdcMicrophone;
        use capsules::virtual_adc::AdcDevice;
        use core::mem::MaybeUninit;
        use kernel::hil::adc::Adc;
        use kernel::hil::gpio::Pin;

        static mut BUFFER: [u16; $LEN] = [0; $LEN];

        let mut adc_microphone_adc: &'static capsules::virtual_adc::AdcDevice<'static, $A> =
            components::adc::AdcComponent::new($adc_mux, $channel)
                .finalize(components::adc_component_helper!($A));
        static mut adc_microphone: MaybeUninit<AdcMicrophone<'static, $P>> = MaybeUninit::uninit();
        (
            &mut adc_microphone_adc,
            $pin,
            &mut BUFFER,
            &mut adc_microphone,
        )
    };};
}

pub struct AdcMicrophoneComponent<A: 'static + adc::Adc, P: 'static + gpio::Pin> {
    _adc: PhantomData<A>,
    _pin: PhantomData<P>,
}

impl<A: 'static + adc::Adc, P: 'static + gpio::Pin> AdcMicrophoneComponent<A, P> {
    pub fn new() -> AdcMicrophoneComponent<A, P> {
        AdcMicrophoneComponent {
            _adc: PhantomData,
            _pin: PhantomData,
        }
    }
}

impl<A: 'static + adc::Adc, P: 'static + gpio::Pin> Component for AdcMicrophoneComponent<A, P> {
    type StaticInput = (
        &'static AdcDevice<'static, A>,
        Option<&'static P>,
        &'static mut [u16],
        &'static mut MaybeUninit<AdcMicrophone<'static, P>>,
    );
    type Output = &'static AdcMicrophone<'static, P>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let adc_microphone = static_init_half!(
            static_buffer.3,
            AdcMicrophone<'static, P>,
            AdcMicrophone::new(static_buffer.0, static_buffer.1, static_buffer.2)
        );

        static_buffer.0.set_client(adc_microphone);

        adc_microphone
    }
}
