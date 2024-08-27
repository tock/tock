// Copyright OxidOS Automotive 2024.

use crate::peripherals::gpio;

/// Types of LEDs. In OxidOS, these are the structs that actually
/// wrap the low level Pin driver.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum LedType {
    /// LEDs in which on is when GPIO is high.
    LedHigh,
    /// LEDs in which on is when GPIO is low.
    LedLow,
}

/// The [`Led`] capsule can be configured through the GPIO pins that are used by the capsule and
/// the type of LED that they're configured as.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Led<G: gpio::Gpio> {
    /// The type of LED the pins are wrapped in.
    inner: LedType,
    /// Pins that are used by the capsule.
    pins: Vec<G::PinId>,
}

impl<G: gpio::Gpio> Led<G> {
    /// Create a new [`Led`] instance.
    pub fn new(inner: LedType, pins: Vec<G::PinId>) -> Self {
        Led { inner, pins }
    }

    pub fn add_pin(&mut self, pin: G::PinId) {
        self.pins.push(pin);
    }

    pub fn add_pins(&mut self, pins: &mut Vec<G::PinId>) {
        self.pins.append(pins);
    }

    pub fn set_inner(&mut self, inner: LedType) {
        self.inner = inner;
    }

    pub fn is_empty(&self) -> bool {
        self.pins.is_empty()
    }
}
