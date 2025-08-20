// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Tools for adapting to different screen formats.

use core::cell::Cell;

use kernel::hil::screen::{Screen, ScreenClient, ScreenPixelFormat, ScreenRotation};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

/// Convert an ARGB8888 formatted screen to a Mono8BitPage formatted screen of
/// the same resolution.
///
/// All pixels are converted to mono (black and white)
pub struct ScreenARGB8888ToMono8BitPage<'a, S: Screen<'a>> {
    screen: &'a S,
    draw_buffer: OptionalCell<SubSliceMut<'static, u8>>,
    draw_width: Cell<usize>,
    client_buffer: OptionalCell<SubSliceMut<'static, u8>>,
    client: OptionalCell<&'a dyn ScreenClient>,
}

impl<'a, S: Screen<'a>> ScreenARGB8888ToMono8BitPage<'a, S> {
    pub fn new(screen: &'a S, draw_buffer: &'static mut [u8]) -> Self {
        assert!(draw_buffer.len() % 4 == 0);

        ScreenARGB8888ToMono8BitPage {
            screen,
            draw_buffer: OptionalCell::new(SubSliceMut::new(draw_buffer)),
            draw_width: Cell::new(0),
            client_buffer: OptionalCell::empty(),
            client: OptionalCell::empty(),
        }
    }
}

struct EightRowColumnPixelIter<'a> {
    buf: &'a mut [u8],
    width: usize,
    row: usize,
    col: usize,
}

impl<'a> EightRowColumnPixelIter<'a> {
    pub fn new(buf: &'a mut [u8], width: usize) -> Self {
        EightRowColumnPixelIter {
            buf,
            width,
            row: 0,
            col: 0,
        }
    }

    // When trying to implement this as an iterator, we get lifetime issues:
    //
    // impl<'a> Iterator for EightRowColumnPixelIter<'a> {
    //     type Item = &'a mut [u8];
    //     ...
    // }
    fn next(&mut self) -> Option<&mut [u8]> {
        let pixel_offset = self
            .row
            .checked_mul(self.width)
            .and_then(|off| off.checked_add(self.col))
            .and_then(|off| off.checked_mul(4))?;

        self.row += 1;
        if self.row % 8 == 0 {
            self.row -= 8;
            self.col += 1;
        }
        if self.col == self.width {
            self.col = 0;
            self.row += 8;
        }

        self.buf.get_mut(pixel_offset..(pixel_offset + 4))
    }
}

impl<'a, S: Screen<'a>> Screen<'a> for ScreenARGB8888ToMono8BitPage<'a, S> {
    fn set_client(&self, client: &'a dyn ScreenClient) {
        self.client.replace(client);
    }

    fn get_resolution(&self) -> (usize, usize) {
        self.screen.get_resolution()
    }

    fn get_pixel_format(&self) -> ScreenPixelFormat {
        ScreenPixelFormat::Mono_8BitPage
    }

    fn get_rotation(&self) -> ScreenRotation {
        self.screen.get_rotation()
    }

    fn set_write_frame(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<(), ErrorCode> {
        // We can only write 8 full rows at a time:
        if y % 8 != 0 || height % 8 != 0 {
            return Err(ErrorCode::INVAL);
        }

        self.draw_width.set(width);
        self.screen.set_write_frame(x, y, width, height)
    }

    fn write(
        &self,
        buffer: SubSliceMut<'static, u8>,
        continue_write: bool,
    ) -> Result<(), ErrorCode> {
        fn into_bits(byte: u8) -> [bool; 8] {
            let mut dst = [false; 8];
            for (i, d) in dst.iter_mut().enumerate() {
                *d = (byte & (1 << i)) != 0;
            }
            dst
        }

        let Some(mut draw_buffer) = self.draw_buffer.take() else {
            return Err(ErrorCode::BUSY);
        };

        draw_buffer.reset();

        // For each bit in the client buffer, we require 4 bytes in the draw
        // buffer. So, for a full byte in the source, that's 32 bytes in the
        // draw buffer:
        let mut bytes_written = 0;
        let mut dst_iter =
            EightRowColumnPixelIter::new(draw_buffer.as_mut_slice(), self.draw_width.get());
        for src_mono_8bit_page in buffer.as_slice().iter() {
            // Now, write an "8 set" of rows:
            for v in into_bits(*src_mono_8bit_page) {
                dst_iter.next().unwrap().copy_from_slice(&[
                    0x00,
                    0xFF * (v as u8),
                    0xFF * (v as u8),
                    0xFF * (v as u8),
                ]);
                bytes_written += 4;
            }
        }
        draw_buffer.slice(..bytes_written);

        // Now, draw this buffer:
        assert!(self.client_buffer.replace(buffer).is_none());
        self.screen.write(draw_buffer, continue_write)
    }

    fn set_brightness(&self, brightness: u16) -> Result<(), ErrorCode> {
        self.screen.set_brightness(brightness)
    }

    fn set_power(&self, enabled: bool) -> Result<(), ErrorCode> {
        self.screen.set_power(enabled)
    }

    fn set_invert(&self, enabled: bool) -> Result<(), ErrorCode> {
        self.screen.set_invert(enabled)
    }
}

impl<'a, S: Screen<'a>> ScreenClient for ScreenARGB8888ToMono8BitPage<'a, S> {
    fn command_complete(&self, result: Result<(), ErrorCode>) {
        self.client.map(|c| c.command_complete(result));
    }

    fn write_complete(&self, buffer: SubSliceMut<'static, u8>, _result: Result<(), ErrorCode>) {
        self.draw_buffer.replace(buffer);
        self.client
            .map(|c| c.write_complete(self.client_buffer.take().unwrap(), Ok(())));
    }

    fn screen_is_ready(&self) {
        self.client.map(|c| c.screen_is_ready());
    }
}
