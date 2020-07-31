//! Component for the ADC on the imix board.
//!
//! This provides one Component, AdcComponent, which implements the
//! dedicated userspace syscall interface to the SAM4L ADC. It
//! provides 6 ADC channels, AD0-AD5.
//!
//! Usage
//! -----
//! ```rust
//! let adc = AdcComponent::new().finalize(());
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::adc;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;
use sam4l::adc::Channel;

pub struct AdcComponent {
    board_kernel: &'static kernel::Kernel,
    adc: &'static sam4l::adc::Adc,
}

impl AdcComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        adc: &'static sam4l::adc::Adc,
    ) -> AdcComponent {
        AdcComponent { board_kernel, adc }
    }
}

impl Component for AdcComponent {
    type StaticInput = ();
    type Output = &'static adc::AdcDedicated<'static, sam4l::adc::Adc>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let adc_channels = static_init!(
            [sam4l::adc::AdcChannel; 6],
            [
                sam4l::adc::AdcChannel::new(Channel::AD1), // AD0
                sam4l::adc::AdcChannel::new(Channel::AD2), // AD1
                sam4l::adc::AdcChannel::new(Channel::AD3), // AD2
                sam4l::adc::AdcChannel::new(Channel::AD4), // AD3
                sam4l::adc::AdcChannel::new(Channel::AD5), // AD4
                sam4l::adc::AdcChannel::new(Channel::AD6), // AD5
            ]
        );
        // Capsule expects references inside array bc it was built assuming model in which
        // global structs are used, so this is a bit of a hack to pass it what it wants.
        let ref_channels = static_init!(
            [&sam4l::adc::AdcChannel; 6],
            [
                &adc_channels[0],
                &adc_channels[1],
                &adc_channels[2],
                &adc_channels[3],
                &adc_channels[4],
                &adc_channels[5],
            ]
        );
        let adc = static_init!(
            adc::AdcDedicated<'static, sam4l::adc::Adc>,
            adc::AdcDedicated::new(
                &self.adc,
                self.board_kernel.create_grant(&grant_cap),
                ref_channels,
                &mut adc::ADC_BUFFER1,
                &mut adc::ADC_BUFFER2,
                &mut adc::ADC_BUFFER3
            )
        );
        self.adc.set_client(adc);

        adc
    }
}
