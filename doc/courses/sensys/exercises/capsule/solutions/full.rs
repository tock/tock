//! Sample capsule for Tock course at SOSP. It handles an alarm to
//! sample the ambient light sensor.

#![feature(const_fn,const_cell_new)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

use kernel::hil::time::{self, Alarm, Frequency};
use kernel::hil::sensors::{AmbientLight, AmbientLightClient};

pub struct Sensys<'a, A: Alarm + 'a>  {
    alarm: &'a A,
    light: &'a AmbientLight,
}

impl<'a, A: Alarm> Sensys<'a, A> {
    pub fn new(alarm: &'a A, light: &'a AmbientLight) -> Sensys<'a, A> {
        Sensys {
           alarm: alarm,
           light: light,
        }
    }

    pub fn start(&self) {
        self.alarm.set_alarm(
            self.alarm.now().wrapping_add(<A::Frequency>::frequency()));
    }
}

impl<'a, A: Alarm> time::Client for Sensys<'a, A> {
    fn fired(&self) {
        self.light.read_light_intensity();
    }
}

impl<'a, A: Alarm> AmbientLightClient for Sensys<'a, A> {
    fn callback(&self, lux: usize) {
        debug!("Light reading: {}", lux);
        self.start();
    }
}

