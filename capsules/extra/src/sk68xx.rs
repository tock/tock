// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Single-wire LED.
//!
//! Tested with the SK68XXMINI on the ESP32-C3-DevKitM-1.
//!
//! Datasheet: <https://www.rose-lighting.com/wp-content/uploads/sites/53/2020/05/SK68XX-MINI-HS-REV.04-EN23535RGB-thick.pdf>

use core::cell::Cell;

use kernel::hil::gpio::Pin;
use kernel::hil::led::Led;

/// The single-wire, tri-color (RGB) LED.
///
/// The pulses need to be calibrated based on the clock speed of the chip.
/// - `T0H`: Number of nops needed for about 0.28 us. This is then scaled for T0L, T1H, and T1L.
pub struct Sk68xx<'a, P: Pin, const T0H: usize> {
    pin: &'a P,
    nop: fn(),
    red: Cell<bool>,
    green: Cell<bool>,
    blue: Cell<bool>,
}

impl<'a, P: Pin, const T0H: usize> Sk68xx<'a, P, T0H> {
    pub fn new(pin: &'a P, nop: fn()) -> Self {
        Self {
            pin,
            nop,
            red: Cell::new(false),
            green: Cell::new(false),
            blue: Cell::new(false),
        }
    }

    pub fn init(&self) {
        self.pin.make_output();
        self.pin.clear();
        for _ in 0..1000 {
            (self.nop)();
        }
    }

    fn write(&self) {
        let red = self.red.get();
        let green = self.green.get();
        let blue = self.blue.get();

        for i in 0..24 {
            let high = if i < 8 {
                // Green
                green
            } else if i < 16 {
                // Red
                red
            } else {
                // Blue
                blue
            };
            if high {
                // High for 0.74 us
                self.pin.set();
                for _ in 0..(T0H * 3) {
                    (self.nop)();
                }

                // Low for 0.52 us
                self.pin.clear();
                for _ in 0..(T0H * 2) {
                    (self.nop)();
                }
            } else {
                // High for 0.28 us
                self.pin.set();
                for _ in 0..T0H {
                    (self.nop)();
                }

                // Low for 0.94 us
                self.pin.clear();
                for _ in 0..(T0H * 4) {
                    (self.nop)();
                }
            }
        }

        // Rest period after to reset the writing and permit a subsequent write
        // to the LED.
        for _ in 0..1000 {
            (self.nop)();
        }
    }

    fn update_red(&self, on: bool) {
        self.red.set(on);
        self.write();
    }

    fn update_green(&self, on: bool) {
        self.green.set(on);
        self.write();
    }

    fn update_blue(&self, on: bool) {
        self.blue.set(on);
        self.write();
    }

    fn toggle_red(&self) {
        self.red.set(!self.red.get());
        self.write();
    }

    fn toggle_green(&self) {
        self.green.set(!self.green.get());
        self.write();
    }

    fn toggle_blue(&self) {
        self.blue.set(!self.blue.get());
        self.write();
    }

    fn get_red(&self) -> bool {
        self.red.get()
    }

    fn get_green(&self) -> bool {
        self.green.get()
    }

    fn get_blue(&self) -> bool {
        self.blue.get()
    }
}

// One of the LEDs on the tri-color LED.
//
// - Index 0: Red
// - Index 1: Green
// - Index 2: Blue
pub struct Sk68xxLed<'a, P: Pin, const T0H: usize> {
    sk68xx: &'a Sk68xx<'a, P, T0H>,
    index: usize,
}

impl<'a, P: Pin, const T0H: usize> Sk68xxLed<'a, P, T0H> {
    pub fn new(sk68xx: &'a Sk68xx<'a, P, T0H>, index: usize) -> Self {
        Self { sk68xx, index }
    }
}

impl<'a, P: Pin, const T0H: usize> Led for Sk68xxLed<'a, P, T0H> {
    fn init(&self) {}

    fn on(&self) {
        match self.index {
            0 => self.sk68xx.update_red(true),
            1 => self.sk68xx.update_green(true),
            2 => self.sk68xx.update_blue(true),
            _ => {}
        }
    }

    fn off(&self) {
        match self.index {
            0 => self.sk68xx.update_red(false),
            1 => self.sk68xx.update_green(false),
            2 => self.sk68xx.update_blue(false),
            _ => {}
        }
    }

    fn toggle(&self) {
        match self.index {
            0 => self.sk68xx.toggle_red(),
            1 => self.sk68xx.toggle_green(),
            2 => self.sk68xx.toggle_blue(),
            _ => {}
        }
    }

    fn read(&self) -> bool {
        match self.index {
            0 => self.sk68xx.get_red(),
            1 => self.sk68xx.get_green(),
            2 => self.sk68xx.get_blue(),
            _ => false,
        }
    }
}
