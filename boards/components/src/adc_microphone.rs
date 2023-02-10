//! Component for ADC Microphone
//!
//! Usage
//! -----
//!
//!
//! ```rust
//! let adc_microphone = components::adc_microphone::AdcMicrophoneComponent::new(
//!     adc_mux,
//!     nrf52833::adc::AdcChannelSetup::setup(
//!         nrf52833::adc::AdcChannel::AnalogInput3,
//!         nrf52833::adc::AdcChannelGain::Gain4,
//!         nrf52833::adc::AdcChannelResistor::Bypass,
//!         nrf52833::adc::AdcChannelResistor::Pulldown,
//!         nrf52833::adc::AdcChannelSamplingTime::us3,
//!     ),
//!     Some(&nrf52833_peripherals.gpio_port[LED_MICROPHONE_PIN]),
//! )
//! .finalize(components::adc_microphone_component_static!(
//!     // adc
//!     nrf52833::adc::Adc,
//!     // buffer size
//!     50,
//!     // gpio
//!     nrf52833::gpio::GPIOPin
//! ));
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_adc::AdcDevice;
use extra_capsules::adc_microphone::AdcMicrophone;
use kernel::component::Component;
use kernel::hil::adc::{self, AdcChannel};
use kernel::hil::gpio;

#[macro_export]
macro_rules! adc_microphone_component_static {
    ($A:ty, $LEN:literal, $P: ty $(,)?) => {{
        let adc_device = components::adc_component_static!($A);
        let buffer = kernel::static_buf!([u16; $LEN]);
        let adc_microphone =
            kernel::static_buf!(extra_capsules::adc_microphone::AdcMicrophone<'static, $P>);

        (adc_device, buffer, adc_microphone)
    };};
}

pub struct AdcMicrophoneComponent<
    A: 'static + adc::Adc,
    P: 'static + gpio::Pin,
    const BUF_LEN: usize,
> {
    adc_mux: &'static core_capsules::virtual_adc::MuxAdc<'static, A>,
    adc_channel: A::Channel,
    pin: Option<&'static P>,
}

impl<A: 'static + adc::Adc, P: 'static + gpio::Pin, const BUF_LEN: usize>
    AdcMicrophoneComponent<A, P, BUF_LEN>
{
    pub fn new(
        adc_mux: &'static core_capsules::virtual_adc::MuxAdc<'static, A>,
        adc_channel: A::Channel,
        pin: Option<&'static P>,
    ) -> AdcMicrophoneComponent<A, P, BUF_LEN> {
        AdcMicrophoneComponent {
            adc_mux,
            adc_channel,
            pin,
        }
    }
}

impl<A: 'static + adc::Adc, P: 'static + gpio::Pin, const BUF_LEN: usize> Component
    for AdcMicrophoneComponent<A, P, BUF_LEN>
{
    type StaticInput = (
        &'static mut MaybeUninit<AdcDevice<'static, A>>,
        &'static mut MaybeUninit<[u16; BUF_LEN]>,
        &'static mut MaybeUninit<AdcMicrophone<'static, P>>,
    );
    type Output = &'static AdcMicrophone<'static, P>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let adc_device =
            crate::adc::AdcComponent::new(self.adc_mux, self.adc_channel).finalize(s.0);

        let buffer = s.1.write([0; BUF_LEN]);

        let adc_microphone = s.2.write(AdcMicrophone::new(adc_device, self.pin, buffer));

        adc_device.set_client(adc_microphone);

        adc_microphone
    }
}
