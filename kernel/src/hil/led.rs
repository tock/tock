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
pub struct LedHigh<'a> {
    pub pin: &'a mut dyn gpio::Pin,
}

/// For LEDs in which on is when GPIO is low.
pub struct LedLow<'a> {
    pub pin: &'a mut dyn gpio::Pin,
}

impl LedHigh<'a> {
    pub fn new(p: &'a mut dyn gpio::Pin) -> LedHigh {
        LedHigh { pin: p }
    }
}

impl LedLow<'a> {
    pub fn new(p: &'a mut dyn gpio::Pin) -> LedLow {
        LedLow { pin: p }
    }
}

impl Led for LedHigh<'a> {
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

impl Led for LedLow<'a> {
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
