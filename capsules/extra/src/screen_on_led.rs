// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Display virtual LEDs on a screen.
//!
//! This creates virtual LEDs by drawing boxes on the screen. The output looks
//! roughly like this:
//!
//! ```text
//!     ┌─┐┌─┐┌─┐┌─┐
//! LED └─┘└─┘└─┘└─┘
//! ```
//! The boxes get filled in if the LED is on.
//!
//! ## Caveat
//!
//! The LED API is based on the GPIO API which is synchronous. Writing to a
//! screen is not. This means LED events can easily get lost if the screen
//! operation hasn't finished yet.

use core::cell::Cell;
use kernel::hil;
use kernel::utilities::cells::MapCell;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

/// How many pixels of padding on the left and right side of the graphic.
const LEFT_RIGTH_PADDING: usize = 2;
/// How many pixels of padding above and below the graphic.
const TOP_BOTTOM_PADDING: usize = 2;
/// How many pixels of padding between the text "LED" and the squares.
const TEXT_LEDS_PADDING: usize = 2;
/// How many pixels of padding between each letter "L", "E", "D".
const TEXT_SPACING: usize = 2;
/// How many pixels of padding above and below the "LED" text.
const TEXT_TOP_BOTTOM_PADDING: usize = 4;

/// Mirror of the [`kernel::hil::led::Led`] trait but that supports LED indices.
///
/// This is implemented by [`ScreenOnLed`]. The trait allows us to avoid having
/// to replicate the `const` types on [`ScreenOnLed`].
pub trait LedIndexed {
    fn init(&self, index: usize);
    fn on(&self, index: usize);
    fn off(&self, index: usize);
    fn toggle(&self, index: usize);
    fn read(&self, index: usize) -> bool;
}

/// A simple wrapper type to contain the index of the LED.
///
/// Since we don't have a single GPIO pin to use, we instead store the index
/// and call into the shared [`ScreenOnLed`].
pub struct ScreenOnLedSingle<'a, L: LedIndexed> {
    /// Essentially only [`ScreenOnLed`].
    led_controller: &'a L,
    /// Which LED this is.
    index: usize,
}

impl<'a, L: LedIndexed> ScreenOnLedSingle<'a, L> {
    pub fn new(led_controller: &'a L, index: usize) -> Self {
        Self {
            led_controller,
            index,
        }
    }
}

impl<L: LedIndexed> hil::led::Led for ScreenOnLedSingle<'_, L> {
    fn init(&self) {
        self.led_controller.init(self.index);
    }

    fn on(&self) {
        self.led_controller.on(self.index);
    }

    fn off(&self) {
        self.led_controller.off(self.index);
    }

    fn toggle(&self) {
        self.led_controller.toggle(self.index);
    }

    fn read(&self) -> bool {
        self.led_controller.read(self.index)
    }
}

pub struct ScreenOnLed<
    'a,
    S: hil::screen::Screen<'a>,
    const NUM_LEDS: usize,
    const SCREEN_WIDTH: usize,
    const SCREEN_HEIGHT: usize,
> {
    /// Underlying screen driver to use.
    screen: &'a S,

    /// Array of the state of each LED. Needed for toggle() and read().
    leds: Cell<[bool; NUM_LEDS]>,

    /// Buffer to render the LED graphics into.
    buffer: MapCell<&'static mut [u8]>,

    /// Whether or not the buffer is initialized with the LED graphics.
    initialized: Cell<bool>,

    /// Whether LEDs were changed while a screen write was outstanding. When the
    /// write finishes, write again with the updated LED state.
    dirty: Cell<bool>,
}

impl<
        'a,
        S: hil::screen::Screen<'a>,
        const NUM_LEDS: usize,
        const SCREEN_WIDTH: usize,
        const SCREEN_HEIGHT: usize,
    > ScreenOnLed<'a, S, NUM_LEDS, SCREEN_WIDTH, SCREEN_HEIGHT>
{
    pub const fn new(screen: &'a S, buffer: &'static mut [u8]) -> Self {
        Self {
            screen,
            leds: Cell::new([false; NUM_LEDS]),
            buffer: MapCell::new(buffer),
            initialized: Cell::new(false),
            dirty: Cell::new(false),
        }
    }

    /// Draw the main LED graphic (e.g. the text and LED boxes).
    fn initialize_leds(&self) {
        self.buffer.take().map(|buffer| {
            self.render(buffer);
            let data = SubSliceMut::new(buffer);
            let _ = self.screen.write(data, false);
        });
    }

    /// Draw all LEDs.
    ///
    /// This is a hack to help correctly show LEDs even though the screen is
    /// async and LEDs are sync. We can get LEDs changing when the buffer is
    /// being used by the screen, so we try to hide that by updating the status
    /// of all LEDs each time.
    fn show_leds(&self) {
        if !self.initialized.get() {
            return;
        }

        self.buffer.take().map_or_else(
            || {
                // We can't update the LEDs because we don't have the screen
                // buffer. This means a screen write is in progress. We mark
                // this and re-write when the current screen write finishes.
                self.dirty.set(true);
            },
            |buffer| {
                let leds = self.leds.get();
                for (i, led_state) in leds.iter().enumerate() {
                    self.render_led_state(buffer, i, *led_state);
                }
                let data = SubSliceMut::new(buffer);
                let _ = self.screen.write(data, false);
            },
        );
    }

    fn get_led_offset(&self, led_index: usize) -> usize {
        let led_dimension = self.get_size().1;

        LEFT_RIGTH_PADDING
            + self.get_led_width(led_dimension)
            + TEXT_LEDS_PADDING
            + ((led_dimension + 1) * led_index)
    }

    fn render(&self, buffer: &mut [u8]) {
        self.render_led_text(buffer, LEFT_RIGTH_PADDING);
        for i in 0..NUM_LEDS {
            self.render_led(buffer, i);
        }
    }

    fn render_led_text(&self, buffer: &mut [u8], x_offset: usize) {
        let y_offset = self.get_size().2;

        let y_top = TOP_BOTTOM_PADDING + TEXT_TOP_BOTTOM_PADDING + y_offset;
        let y_bottom = SCREEN_HEIGHT - TOP_BOTTOM_PADDING - TEXT_TOP_BOTTOM_PADDING - y_offset;
        let height = y_bottom - y_top;

        // L
        let l_offset = x_offset;
        let l_width = self.get_char_width(height, 'l');
        self.write_vertical_line(buffer, l_offset, y_top, height, 1);
        self.write_horizontal_line(buffer, l_offset, y_bottom, l_width, 1);

        // E
        let e_offset = x_offset + l_width + TEXT_SPACING;
        let e_width = self.get_char_width(height, 'e');
        self.write_vertical_line(buffer, e_offset, y_top, height, 1);
        self.write_horizontal_line(buffer, e_offset, y_top, e_width, 1);
        self.write_horizontal_line(buffer, e_offset, y_bottom, e_width, 1);
        self.write_horizontal_line(
            buffer,
            e_offset,
            usize::midpoint(y_top, y_bottom),
            e_width / 2,
            1,
        );

        // D
        let d_offset = e_offset + e_width + TEXT_SPACING;
        let d_width = self.get_char_width(height, 'd');
        self.write_vertical_line(buffer, d_offset, y_top, height, 1);
        self.write_vertical_line(buffer, d_offset + d_width, y_top, height, 1);
        self.write_horizontal_line(buffer, d_offset, y_top, d_width, 1);
        self.write_horizontal_line(buffer, d_offset, y_bottom, d_width, 1);
    }

    fn render_led(&self, buffer: &mut [u8], led_index: usize) {
        // Draw two squares, one on, then one inside that is off.

        let (_width, led_dimension, y_offset) = self.get_size();
        let x_offset: usize = self.get_led_offset(led_index);

        // Write the outside box fully on.
        self.write_square(
            buffer.as_mut(),
            x_offset,
            TOP_BOTTOM_PADDING + y_offset,
            led_dimension,
            1,
        );
        // Clear the inside to make just the border.
        self.write_square(
            buffer.as_mut(),
            x_offset + 1,
            TOP_BOTTOM_PADDING + y_offset + 1,
            led_dimension - 2,
            0,
        );
    }

    fn render_led_state(&self, buffer: &mut [u8], led_index: usize, on: bool) {
        let (_width, led_dimension, y_offset) = self.get_size();
        let x_offset: usize = self.get_led_offset(led_index);

        // Clear the inside to make just the border.
        self.write_square(
            buffer.as_mut(),
            x_offset + 1,
            TOP_BOTTOM_PADDING + y_offset + 1,
            led_dimension - 2,
            0,
        );

        if on {
            // Draw the LED as on.
            self.write_square(
                buffer.as_mut(),
                x_offset + 2,
                TOP_BOTTOM_PADDING + y_offset + 2,
                led_dimension - 4,
                1,
            );
        }
    }

    fn write_square(&self, buffer: &mut [u8], x: usize, y: usize, dimension: usize, val: usize) {
        for i in 0..dimension {
            for j in 0..dimension {
                let pixel_x = i + x;
                let pixel_y = j + y;
                let byte = ((pixel_y / 8) * SCREEN_WIDTH) + pixel_x;
                let bit = pixel_y % 8;
                if val & 0x1 == 0x1 {
                    buffer[byte] |= 1 << bit;
                } else {
                    buffer[byte] &= !(1 << bit);
                }
            }
        }
    }

    fn write_horizontal_line(
        &self,
        buffer: &mut [u8],
        x: usize,
        y: usize,
        length: usize,
        val: usize,
    ) {
        for i in 0..length {
            let pixel_x = i + x;
            let byte = ((y / 8) * SCREEN_WIDTH) + pixel_x;
            let bit = y % 8;
            if val & 0x1 == 0x1 {
                buffer[byte] |= 1 << bit;
            } else {
                buffer[byte] &= !(1 << bit);
            }
        }
    }

    fn write_vertical_line(
        &self,
        buffer: &mut [u8],
        x: usize,
        y: usize,
        length: usize,
        val: usize,
    ) {
        for i in 0..length {
            let pixel_y = i + y;
            let byte = ((pixel_y / 8) * SCREEN_WIDTH) + x;
            let bit = pixel_y % 8;
            if val & 0x1 == 0x1 {
                buffer[byte] |= 1 << bit;
            } else {
                buffer[byte] &= !(1 << bit);
            }
        }
    }

    /// Find a size of the graphic that works with the screen by shrinking the
    /// LEDs until everything fits.
    pub const fn get_size(&self) -> (usize, usize, usize) {
        let mut width = SCREEN_WIDTH + 1;
        let mut led_dimension = SCREEN_HEIGHT - (TOP_BOTTOM_PADDING * 2);

        while width > SCREEN_WIDTH {
            // Shrink LEDs by 1 pixel.
            led_dimension -= 1;

            let leds_width: usize = (led_dimension * NUM_LEDS) + (NUM_LEDS - 1);
            width = LEFT_RIGTH_PADDING
                + self.get_led_width(led_dimension)
                + TEXT_LEDS_PADDING
                + leds_width
                + LEFT_RIGTH_PADDING;
        }

        let y_offset = (SCREEN_HEIGHT - (TOP_BOTTOM_PADDING * 2) - led_dimension) / 2;

        (width, led_dimension, y_offset)
    }

    pub const fn get_char_width(&self, height: usize, c: char) -> usize {
        match c {
            'l' => (height * 3) / 5,
            'e' => (height * 3) / 5,
            'd' => (height * 2) / 4,
            _ => 0,
        }
    }

    pub const fn get_led_width(&self, height: usize) -> usize {
        let height = height - (TEXT_TOP_BOTTOM_PADDING * 2);

        let l_width = self.get_char_width(height, 'l');
        let e_width = self.get_char_width(height, 'e');
        let d_width = self.get_char_width(height, 'd');
        l_width + TEXT_SPACING + e_width + TEXT_SPACING + d_width
    }
}

impl<
        'a,
        S: hil::screen::Screen<'a>,
        const NUM_LEDS: usize,
        const SCREEN_WIDTH: usize,
        const SCREEN_HEIGHT: usize,
    > LedIndexed for ScreenOnLed<'a, S, NUM_LEDS, SCREEN_WIDTH, SCREEN_HEIGHT>
{
    fn init(&self, _index: usize) {}

    fn on(&self, index: usize) {
        let mut leds = self.leds.get();
        leds[index] = true;
        self.leds.set(leds);
        self.show_leds();
    }

    fn off(&self, index: usize) {
        let mut leds = self.leds.get();
        leds[index] = false;
        self.leds.set(leds);
        self.show_leds();
    }

    fn toggle(&self, index: usize) {
        let mut leds = self.leds.get();
        let updated = !leds[index];
        leds[index] = updated;
        self.leds.set(leds);
        self.show_leds();
    }

    fn read(&self, index: usize) -> bool {
        self.leds.get()[index]
    }
}

impl<
        'a,
        S: hil::screen::Screen<'a>,
        const NUM_LEDS: usize,
        const SCREEN_WIDTH: usize,
        const SCREEN_HEIGHT: usize,
    > hil::screen::ScreenClient for ScreenOnLed<'a, S, NUM_LEDS, SCREEN_WIDTH, SCREEN_HEIGHT>
{
    fn command_complete(&self, _r: Result<(), ErrorCode>) {}

    fn write_complete(&self, data: SubSliceMut<'static, u8>, _r: Result<(), ErrorCode>) {
        self.buffer.replace(data.take());

        // Check if LED state changed while we were writing. If so, do another
        // screen write to update the LEDs.
        if self.dirty.get() {
            self.dirty.set(false);
            self.show_leds();
        }
    }

    fn screen_is_ready(&self) {
        if !self.initialized.get() {
            self.initialized.set(true);
            self.initialize_leds();
        }
    }
}
