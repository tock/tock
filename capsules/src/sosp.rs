//! Sample capsule for Tock course at SOSP. It handles an alarm to
//! sample the ambient light sensor.

use kernel::hil::time::{self, Alarm, Frequency};
use kernel::hil::sensors::{AmbientLight, AmbientLightClient};

pub struct Sosp<'a, A: Alarm + 'a>  {
    alarm: &'a A,
    light: &'a AmbientLight,
}

impl<'a, A: Alarm> Sosp<'a, A> {
    pub fn new(alarm: &'a A, light: &'a AmbientLight) -> Sosp<'a, A> {
        Sosp {
           alarm: alarm,
           light: light,
        }
    }

    pub fn start(&self) {
    }
}

impl<'a, A: Alarm> time::Client for Sosp<'a, A> {
    fn fired(&self) {
    }
}

impl<'a, A: Alarm> AmbientLightClient for Sosp<'a, A> {
    fn callback(&self, lux: usize) {
    }
}
