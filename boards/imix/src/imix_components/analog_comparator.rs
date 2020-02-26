//! Component for the Analog Comparator on the imix board.
//!
//! This provides one Component, AcComponent, which implements
//! a userspace syscall interface to the SAM4L ACIFC. It provides
//! 4 AC channels, AC0-AC3.
//!
//! Usage
//! -----
//! ```rust
//! let ac = AcComponent::new().finalize(());
//! ```

// Author: Danilo Verhaert <verhaert@cs.stanford.edu>
// Last modified: August 7th, 2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::analog_comparator;
use kernel::component::Component;
use kernel::static_init;

pub struct AcComponent<'ker> {
    _lifetime: core::marker::PhantomData<&'ker ()>,
}

impl AcComponent<'ker> {
    pub fn new() -> AcComponent<'ker> {
        AcComponent {
            _lifetime: core::marker::PhantomData,
        }
    }
}

impl Component for AcComponent<'ker>
where
    'ker: 'static,
{
    type StaticInput = ();
    type Output =
        &'static analog_comparator::AnalogComparator<'static, 'ker, sam4l::acifc::Acifc<'static>>;

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        let ac_channels = static_init!(
            [&'static sam4l::acifc::AcChannel; 4],
            [
                &sam4l::acifc::CHANNEL_AC0,
                &sam4l::acifc::CHANNEL_AC1,
                &sam4l::acifc::CHANNEL_AC2,
                &sam4l::acifc::CHANNEL_AC3,
            ]
        );
        let analog_comparator = static_init!(
            analog_comparator::AnalogComparator<'static, '_, sam4l::acifc::Acifc>,
            analog_comparator::AnalogComparator::new(&mut sam4l::acifc::ACIFC, ac_channels)
        );
        sam4l::acifc::ACIFC.set_client(analog_comparator);

        analog_comparator
    }
}
