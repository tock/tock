//! Interface for LEDs that abstract away polarity and pin.
//!
//!  Author: Philip Levis <pal@cs.stanford.edu>
//!  Date: July 31, 2015
//!

use crate::hil::gpio;

pub trait Led {
    fn init(&mut self);
    fn on(&mut self);
    fn off(&mut self);
    fn toggle(&mut self);
    fn read(&self) -> bool;
}

/// For LEDs in which on is when GPIO is high.
pub struct LedHigh<'a, P: gpio::Pin> {
    pub pin: &'a P,
}

/// For LEDs in which on is when GPIO is low.
pub struct LedLow<'a, P: gpio::Pin> {
    pub pin: &'a P,
}

impl<'a, P: gpio::Pin> LedHigh<'a, P> {
    pub fn new(p: &'a P) -> Self {
        Self { pin: p }
    }
}

impl<'a, P: gpio::Pin> LedLow<'a, P> {
    pub fn new(p: &'a P) -> Self {
        Self { pin: p }
    }
}

impl<P: gpio::Pin> Led for LedHigh<'_, P> {
    fn init(&mut self) {
        self.pin.make_output();
    }

    fn on(&mut self) {
        self.pin.set();
    }

    fn off(&mut self) {
        self.pin.clear();
    }

    fn toggle(&mut self) {
        self.pin.toggle();
    }

    fn read(&self) -> bool {
        self.pin.read()
    }
}

impl<P: gpio::Pin> Led for LedLow<'_, P> {
    fn init(&mut self) {
        self.pin.make_output();
    }

    fn on(&mut self) {
        self.pin.clear();
    }

    fn off(&mut self) {
        self.pin.set();
    }

    fn toggle(&mut self) {
        self.pin.toggle();
    }

    fn read(&self) -> bool {
        !self.pin.read()
    }
}
