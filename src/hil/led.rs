/**
 *  led.rs -- Drivers for LEDs that abstract away polarity and pin.
 *
 * Author: Philip Levis <pal@cs.stanford.edu>
 * Date: 7/31/15
 */ 

use gpio;

pub trait Led {
    fn init(&mut self);
    fn on(&mut self);
    fn off(&mut self);
    fn toggle(&mut self);
    fn read(&self) -> bool;
}

/// For LEDs in which on is when GPIO is high.
pub struct LedHigh {
    pub pin: &'static mut gpio::GPIOPin
}

/// For LEDs in which on is when GPIO is low.
pub struct LedLow {
    pub pin: &'static mut gpio::GPIOPin
}

impl LedHigh {
    pub fn new(p: &'static mut gpio::GPIOPin) -> LedHigh {
        LedHigh {
            pin: p
        }
    }
}

impl LedLow {
    pub fn new(p: &'static mut gpio::GPIOPin) -> LedLow {
        LedLow {
            pin: p
        }
    }
}

impl Led for LedHigh {
    fn init(&mut self) {
        self.pin.enable_output();
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

impl Led for LedLow {
    fn init(&mut self) {
        self.pin.enable_output();
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

