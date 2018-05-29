//! Sample capsule for Tock course at SOSP. It handles an alarm to
//! sample the ambient light sensor.

#![feature(const_fn,const_cell_new)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

use core::cell::Cell;
use kernel::common::cells::MapCell;
use kernel::hil::sensors::{AmbientLight, AmbientLightClient};
use kernel::hil::time::{self, Alarm, Frequency};
use kernel::hil::gpio;

const BUFFER_SIZE: usize = 10;

pub struct Sensys<'a, A: Alarm + 'a, L: gpio::Pin + gpio::PinCtl + 'a> {
    alarm: &'a A,
    light: &'a AmbientLight,
    led: &'a L,
    buffer: MapCell<[usize; BUFFER_SIZE]>,
    samples: Cell<usize>,
}

impl<'a, A: Alarm, L: gpio::Pin + gpio::PinCtl + 'a> Sensys<'a, A, L> {
    pub fn new(alarm: &'a A, light: &'a AmbientLight, led: &'a L) -> Sensys<'a, A, L> {
        Sensys {
            alarm: alarm,
            light: light,
            led: led,
            buffer: MapCell::new(Default::default()),
            samples: Cell::new(0),
        }
    }

    pub fn start(&self) {
        let now = self.alarm.now();
        let tenth_of_a_second = <A::Frequency>::frequency() / (BUFFER_SIZE as u32);
        let next_interval = now.wrapping_add(tenth_of_a_second);
        self.alarm.set_alarm(next_interval);
    }
}

impl<'a, A: Alarm, L: gpio::Pin + gpio::PinCtl + 'a> time::Client for Sensys<'a, A, L> {
    fn fired(&self) {
        self.light.read_light_intensity();
        self.start();
    }
}

impl<'a, A: Alarm, L: gpio::Pin + gpio::PinCtl + 'a> AmbientLightClient for Sensys<'a, A, L> {
    fn callback(&self, lux: usize) {
        self.buffer.map(|buf| buf[self.samples.get()] = lux);
        self.samples.set(self.samples.get() + 1);
        if self.samples.get() == BUFFER_SIZE {
            self.buffer.map(|buf| {
                let mut average: usize = 0;
                for v in buf.iter() {
                    average += *v;
                }
                average /= BUFFER_SIZE;
                debug!("Ambient light average: {}, samples: {} {} {} {} {} {} {} {} {} {}",
                       average,
                       buf[0], buf[1], buf[2], buf[3], buf[4],
                       buf[5], buf[6], buf[7], buf[8], buf[9]);
            });
            self.samples.set(0);
        }
        if lux > 100 {
            self.led.set();
        } else {
            self.led.clear();
        }
    }
}
