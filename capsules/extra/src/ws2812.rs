// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! WS2812-compatible single-wire RGB LED (WS2812, SK68XX).
//!
//! Tested with the SK68XXMINI on the ESP32-C3-DevKitM-1 and the WS2812 on the
//! ESP32-C3-DevKit-RUST-2.
//!
//! Datasheet: <https://www.rose-lighting.com/wp-content/uploads/sites/53/2020/05/SK68XX-MINI-HS-REV.04-EN23535RGB-thick.pdf>

use core::cell::Cell;

use kernel::hil::gpio::Pin;
use kernel::hil::led::Led;

/// The single-wire, tri-color (RGB) LED.
///
/// `write_frame` is provided by the chip and sends one 24-bit GRB frame.
pub struct Ws2812<'a, P: Pin> {
    pin: &'a P,
    write_frame: fn(&P, u32),
    red: Cell<bool>,
    green: Cell<bool>,
    blue: Cell<bool>,
}

impl<'a, P: Pin> Ws2812<'a, P> {
    pub fn new(pin: &'a P, write_frame: fn(&P, u32)) -> Self {
        Self {
            pin,
            write_frame,
            red: Cell::new(false),
            green: Cell::new(false),
            blue: Cell::new(false),
        }
    }

    pub fn init(&self) {
        self.pin.make_output();
        self.pin.clear();
    }

    fn write(&self) {
        let byte = |on: bool| if on { 0xFF } else { 0 };
        let grb = byte(self.green.get()) << 16 | byte(self.red.get()) << 8 | byte(self.blue.get());
        (self.write_frame)(self.pin, grb);
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
pub struct Ws2812Led<'a, P: Pin> {
    ws2812: &'a Ws2812<'a, P>,
    index: usize,
}

impl<'a, P: Pin> Ws2812Led<'a, P> {
    pub fn new(ws2812: &'a Ws2812<'a, P>, index: usize) -> Self {
        Self { ws2812, index }
    }
}

impl<P: Pin> Led for Ws2812Led<'_, P> {
    fn init(&self) {}

    fn on(&self) {
        match self.index {
            0 => self.ws2812.update_red(true),
            1 => self.ws2812.update_green(true),
            2 => self.ws2812.update_blue(true),
            _ => {}
        }
    }

    fn off(&self) {
        match self.index {
            0 => self.ws2812.update_red(false),
            1 => self.ws2812.update_green(false),
            2 => self.ws2812.update_blue(false),
            _ => {}
        }
    }

    fn toggle(&self) {
        match self.index {
            0 => self.ws2812.toggle_red(),
            1 => self.ws2812.toggle_green(),
            2 => self.ws2812.toggle_blue(),
            _ => {}
        }
    }

    fn read(&self) -> bool {
        match self.index {
            0 => self.ws2812.get_red(),
            1 => self.ws2812.get_green(),
            2 => self.ws2812.get_blue(),
            _ => false,
        }
    }
}
