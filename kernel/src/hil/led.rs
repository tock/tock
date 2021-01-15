//! Interface for LEDs that abstract away polarity and pin.
//!
//!  Author: Philip Levis <pal@cs.stanford.edu>
//!  Date: July 31, 2015
//!

use crate::hil::gpio;

/// Simple on/off interface for LED pins.
///
/// Since GPIO pins are synchronous in Tock the LED interface is synchronous as
/// well.
pub trait Led {
    /// Initialize the LED. Must be called before the LED is used.
    fn init(&self);

    /// Turn the LED on.
    fn on(&self);

    /// Turn the LED off.
    fn off(&self);

    /// Toggle the LED.
    fn toggle(&self);

    /// Return the on/off state of the LED. `true` if the LED is on, `false` if
    /// it is off.
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
    fn init(&self) {
        self.pin.make_output();
    }

    fn on(&self) {
        self.pin.set();
    }

    fn off(&self) {
        self.pin.clear();
    }

    fn toggle(&self) {
        self.pin.toggle();
    }

    fn read(&self) -> bool {
        self.pin.read()
    }
}

impl<P: gpio::Pin> Led for LedLow<'_, P> {
    fn init(&self) {
        self.pin.make_output();
    }

    fn on(&self) {
        self.pin.clear();
    }

    fn off(&self) {
        self.pin.set();
    }

    fn toggle(&self) {
        self.pin.toggle();
    }

    fn read(&self) -> bool {
        !self.pin.read()
    }
}
