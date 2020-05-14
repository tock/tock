//! An abstraction over the pin multiplexer, nRF5X-family
//!
//! Controller drivers should use the `Pinmux` type (instead of a `u32`) for
//! fields that determine which pins are used by the hardware. The board
//! configuration should create `Pinmux`s and pass them into controller drivers
//! during initialization.

use kernel::common::cells::VolatileCell;

// Note: only the nrf52840 has two ports, but we create two ports to avoid
// gating this code by a feature.
const NUM_PORTS: usize = 2;

const PIN_PER_PORT: usize = 32;

// Keep track of which pins has a `Pinmux` been created for.
static mut USED_PINS: [VolatileCell<u32>; NUM_PORTS] = [VolatileCell::new(0), VolatileCell::new(0)];

/// An opaque wrapper around a configurable pin.
#[derive(Copy, Clone)]
pub struct Pinmux(u32);

impl Pinmux {
    /// Creates a new `Pinmux` wrapping the numbered pin.
    ///
    /// # Panics
    ///
    /// If a `Pinmux` for this pin has already
    /// been created.
    ///
    pub unsafe fn new(pin: u32) -> Pinmux {
        let port: usize = (pin as usize) / PIN_PER_PORT;
        let pin_idx: usize = (pin as usize) % PIN_PER_PORT;
        let used_pins = USED_PINS[port].get();
        if used_pins & (1 << pin_idx) != 0 {
            panic!("Pin {} is already in use!", pin);
        } else {
            USED_PINS[port].set(used_pins | 1 << pin_idx);
            Pinmux(pin)
        }
    }
}

impl Into<u32> for Pinmux {
    fn into(self) -> u32 {
        self.0
    }
}
