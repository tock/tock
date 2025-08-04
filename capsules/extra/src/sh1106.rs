// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! SH1106 OLED Screen Driver
//!
//! This display is similar to the SSD1306, but has two key differences:
//! - The commands are different. In particular, the SH1106 does not support the
//!   `SetColumnAddress` and `SetPageAddress` commands which are useful for
//!   setting frames on the screen.
//! - The driver does not automatically wrap to the next page. This driver
//!   manually sets up each page (row).

use core::cell::Cell;

use crate::ssd1306::Command;
use kernel::hil;
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

// Only need to be able to write one page (row) at a time.
pub const BUFFER_SIZE: usize = 132;

const WIDTH: usize = 128;
const HEIGHT: usize = 64;

// #[derive(Copy, Clone, PartialEq)]
#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    Init,
    SimpleCommand,
    WriteSetPage(u8),
    WritePage(u8),
}

pub struct Sh1106<'a, I: hil::i2c::I2CDevice> {
    i2c: &'a I,
    state: Cell<State>,
    client: OptionalCell<&'a dyn hil::screen::ScreenClient>,
    setup_client: OptionalCell<&'a dyn hil::screen::ScreenSetupClient>,
    buffer: TakeCell<'static, [u8]>,
    write_buffer: MapCell<SubSliceMut<'static, u8>>,
    enable_charge_pump: bool,

    active_frame_x: Cell<u8>,
    active_frame_y: Cell<u8>,
    active_frame_width: Cell<u8>,
    active_frame_height: Cell<u8>,
}

impl<'a, I: hil::i2c::I2CDevice> Sh1106<'a, I> {
    pub fn new(i2c: &'a I, buffer: &'static mut [u8], enable_charge_pump: bool) -> Self {
        Self {
            i2c,
            state: Cell::new(State::Idle),
            client: OptionalCell::empty(),
            setup_client: OptionalCell::empty(),
            buffer: TakeCell::new(buffer),
            write_buffer: MapCell::empty(),
            enable_charge_pump,
            active_frame_x: Cell::new(0),
            active_frame_y: Cell::new(0),
            active_frame_width: Cell::new(0),
            active_frame_height: Cell::new(0),
        }
    }

    pub fn init_screen(&self) {
        let commands = [
            Command::SetDisplayOnOff { on: false },
            Command::SetDisplayClockDivide {
                divide_ratio: 0,
                oscillator_frequency: 0x8,
            },
            Command::SetMultiplexRatio {
                ratio: HEIGHT as u8 - 1,
            },
            Command::SetDisplayOffset { vertical_shift: 0 },
            Command::SetDisplayStartLine { line: 0 },
            Command::SetChargePump {
                enable: self.enable_charge_pump,
            },
            Command::SetMemoryAddressingMode { mode: 0 }, //horizontal
            Command::SetSegmentRemap { reverse: true },
            Command::SetComScanDirection { decrement: true },
            Command::SetComPins {
                alternative: true,
                enable_com: false,
            },
            Command::SetContrast { contrast: 0xcf },
            Command::SetPrechargePeriod {
                phase1: 0x1,
                phase2: 0xf,
            },
            Command::SetVcomDeselect { level: 2 },
            Command::EntireDisplayOn { ignore_ram: false },
            Command::SetDisplayInvert { inverse: false },
            Command::DeactivateScroll,
            Command::SetDisplayOnOff { on: true },
        ];

        match self.send_sequence(&commands) {
            Ok(()) => {
                self.state.set(State::Init);
            }
            Err(_e) => {}
        }
    }

    fn send_sequence(&self, sequence: &[Command]) -> Result<(), ErrorCode> {
        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
            let mut buf_slice = SubSliceMut::new(buffer);

            // Specify this is a series of command bytes.
            buf_slice[0] = 0; // Co = 0, D/C̅ = 0

            // Move the window of the subslice after the command byte header.
            buf_slice.slice(1..);

            for cmd in sequence.iter() {
                cmd.encode(&mut buf_slice);
            }

            // We need the amount of data that has been sliced away
            // at the start of the subslice.
            let remaining_len = buf_slice.len();
            buf_slice.reset();
            let tx_len = buf_slice.len() - remaining_len;

            self.i2c.enable();
            match self.i2c.write(buf_slice.take(), tx_len) {
                Ok(()) => Ok(()),
                Err((_e, buf)) => {
                    self.buffer.replace(buf);
                    self.i2c.disable();
                    Err(ErrorCode::INVAL)
                }
            }
        })
    }

    fn write_continue(&self) -> Result<(), ErrorCode> {
        match self.state.get() {
            State::WriteSetPage(page_index) => {
                self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
                    self.write_buffer.map_or(Err(ErrorCode::NOMEM), |data| {
                        // Calculate which part of the data buffer we need to
                        // write.
                        let start_page_index = self.active_frame_y.get() / 8;
                        let buffer_start_index = ((page_index - start_page_index) as usize)
                            * self.active_frame_width.get() as usize;
                        let page_len = self.active_frame_width.get() as usize;

                        let mut buf_slice = SubSliceMut::new(buffer);

                        // Specify this is data.
                        buf_slice[0] = 0x40; // Co = 0, D/C̅ = 1

                        // Move the window of the subslice after the command
                        // byte header.
                        buf_slice.slice(1..);

                        // Copy the correct page data to the buffer.
                        for i in 0..page_len {
                            buf_slice[i] = data[buffer_start_index + i];
                        }

                        // Length includes the header byte.
                        let tx_len = page_len + 1;

                        self.i2c.enable();
                        match self.i2c.write(buf_slice.take(), tx_len) {
                            Ok(()) => {
                                self.state.set(State::WritePage(page_index));
                                Ok(())
                            }
                            Err((_e, buf)) => {
                                self.buffer.replace(buf);
                                Err(ErrorCode::INVAL)
                            }
                        }
                    })
                })
            }

            State::WritePage(page_index) => {
                // Finished writing a page of data. Check if there is more to
                // do.
                let next_page = page_index + 1;
                let last_page = (self.active_frame_y.get() + self.active_frame_height.get()) / 8;

                if next_page >= last_page {
                    // Done, can issue callback.
                    self.state.set(State::Idle);
                    self.write_buffer.take().map(|buf| {
                        self.client.map(|client| client.write_complete(buf, Ok(())));
                    });
                    Ok(())
                } else {
                    // Continue writing by setting up the next page.
                    self.set_page(next_page)
                }
            }

            _ => Err(ErrorCode::FAIL),
        }
    }

    fn set_page(&self, page_index: u8) -> Result<(), ErrorCode> {
        let column_start = self.active_frame_x.get() + 2;
        let commands = [
            Command::SetPageStartAddress {
                address: page_index,
            },
            Command::SetLowerColumnStartAddress {
                address: column_start,
            },
            Command::SetHigherColumnStartAddress {
                address: column_start,
            },
        ];
        match self.send_sequence(&commands) {
            Ok(()) => {
                self.state.set(State::WriteSetPage(page_index));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl<'a, I: hil::i2c::I2CDevice> hil::screen::ScreenSetup<'a> for Sh1106<'a, I> {
    fn set_client(&self, client: &'a dyn hil::screen::ScreenSetupClient) {
        self.setup_client.set(client);
    }

    fn set_resolution(&self, _resolution: (usize, usize)) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn set_pixel_format(&self, _depth: hil::screen::ScreenPixelFormat) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn set_rotation(&self, _rotation: hil::screen::ScreenRotation) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn get_num_supported_resolutions(&self) -> usize {
        1
    }

    fn get_supported_resolution(&self, index: usize) -> Option<(usize, usize)> {
        match index {
            0 => Some((WIDTH, HEIGHT)),
            _ => None,
        }
    }

    fn get_num_supported_pixel_formats(&self) -> usize {
        1
    }

    fn get_supported_pixel_format(&self, index: usize) -> Option<hil::screen::ScreenPixelFormat> {
        match index {
            0 => Some(hil::screen::ScreenPixelFormat::Mono_8BitPage),
            _ => None,
        }
    }
}

impl<'a, I: hil::i2c::I2CDevice> hil::screen::Screen<'a> for Sh1106<'a, I> {
    fn set_client(&self, client: &'a dyn hil::screen::ScreenClient) {
        self.client.set(client);
    }

    fn get_resolution(&self) -> (usize, usize) {
        (WIDTH, HEIGHT)
    }

    fn get_pixel_format(&self) -> hil::screen::ScreenPixelFormat {
        hil::screen::ScreenPixelFormat::Mono_8BitPage
    }

    fn get_rotation(&self) -> hil::screen::ScreenRotation {
        hil::screen::ScreenRotation::Normal
    }

    fn set_write_frame(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<(), ErrorCode> {
        // Save the current frame settings.
        self.active_frame_x.set(x as u8);
        self.active_frame_y.set(y as u8);
        self.active_frame_width.set(width as u8);
        self.active_frame_height.set(height as u8);

        // The driver RAM is 132 bytes wide, the screen is 128 bytes wide, so we
        // offset by two.
        let column_start: u8 = (x as u8) + 2;
        let commands = [
            Command::SetPageStartAddress {
                address: (y / 8) as u8,
            },
            Command::SetLowerColumnStartAddress {
                address: column_start,
            },
            Command::SetHigherColumnStartAddress {
                address: column_start,
            },
        ];
        match self.send_sequence(&commands) {
            Ok(()) => {
                self.state.set(State::SimpleCommand);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn write(&self, data: SubSliceMut<'static, u8>, _continue: bool) -> Result<(), ErrorCode> {
        self.write_buffer.replace(data);

        // Start by setting the page as active in the screen.
        self.set_page(self.active_frame_y.get() / 8)
    }

    fn set_brightness(&self, brightness: u16) -> Result<(), ErrorCode> {
        let commands = [Command::SetContrast {
            contrast: (brightness >> 8) as u8,
        }];
        match self.send_sequence(&commands) {
            Ok(()) => {
                self.state.set(State::SimpleCommand);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn set_power(&self, enabled: bool) -> Result<(), ErrorCode> {
        let commands = [Command::SetDisplayOnOff { on: enabled }];
        match self.send_sequence(&commands) {
            Ok(()) => {
                self.state.set(State::SimpleCommand);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn set_invert(&self, enabled: bool) -> Result<(), ErrorCode> {
        let commands = [Command::SetDisplayInvert { inverse: enabled }];
        match self.send_sequence(&commands) {
            Ok(()) => {
                self.state.set(State::SimpleCommand);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl<I: hil::i2c::I2CDevice> hil::i2c::I2CClient for Sh1106<'_, I> {
    fn command_complete(&self, buffer: &'static mut [u8], _status: Result<(), hil::i2c::Error>) {
        self.buffer.replace(buffer);
        self.i2c.disable();

        match self.state.get() {
            State::Init => {
                self.state.set(State::Idle);
                self.client.map(|client| client.screen_is_ready());
            }

            State::SimpleCommand => {
                self.state.set(State::Idle);
                self.client.map(|client| client.command_complete(Ok(())));
            }

            State::WritePage(_) | State::WriteSetPage(_) => {
                let _ = self.write_continue();
            }
            _ => {}
        }
    }
}
