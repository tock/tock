// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! SSD1306/SSD1315 OLED Screen

use core::cell::Cell;
use kernel::hil;
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

pub const BUFFER_SIZE: usize = 1032;

const WIDTH: usize = 128;
const HEIGHT: usize = 64;

#[derive(Copy, Clone, PartialEq)]
#[repr(usize)]
pub enum Command {
    // Charge Pump Commands
    /// Charge Pump Setting.
    SetChargePump { enable: bool },

    // Fundamental Commands
    /// SetContrastControl. Double byte command to select 1 out of 256 contrast
    /// steps. Contrast increases as the value increases.
    SetContrast { contrast: u8 },
    /// Entire Display On.
    EntireDisplayOn { ignore_ram: bool },
    /// Set Normal Display.
    SetDisplayInvert { inverse: bool },
    /// Set Display Off.
    SetDisplayOnOff { on: bool },

    // Scrolling Commands
    /// Continuous Horizontal Scroll. Right or Left Horizontal Scroll.
    ContinuousHorizontalScroll {
        left: bool,
        page_start: u8,
        interval: u8,
        page_end: u8,
    },
    /// Continuous Vertical and Horizontal Scroll. Vertical and Right Horizontal
    /// Scroll.
    ContinuousVerticalHorizontalScroll {
        left: bool,
        page_start: u8,
        interval: u8,
        page_end: u8,
        vertical_offset: u8,
    },
    /// Deactivate Scroll. Stop scrolling that is configured by scroll commands.
    DeactivateScroll = 0x2e,
    /// Activate Scroll. Start scrolling that is configured by scroll commands.
    ActivateScroll = 0x2f,
    /// Set Vertical Scroll Area. Set number of rows in top fixed area. The
    /// number of rows in top fixed area is referenced to the top of the GDDRAM
    /// (i.e. row 0).
    SetVerticalScrollArea { rows_fixed: u8, rows_scroll: u8 },

    // Addressing Setting Commands
    /// Set Lower Column Start Address for Page Addressing Mode.
    ///
    /// Set the lower nibble of the column start address register for Page
    /// Addressing Mode using `X[3:0]` as data bits. The initial display line
    /// register is reset to 0000b after RESET.
    SetLowerColumnStartAddress { address: u8 },
    /// Set Higher Column Start Address for Page Addressing Mode.
    ///
    /// Set the higher nibble of the column start address register for Page
    /// Addressing Mode using `X[3:0]` as data bits. The initial display line
    /// register is reset to 0000b after RESET.
    SetHigherColumnStartAddress { address: u8 },
    /// Set Memory Addressing Mode.
    SetMemoryAddressingMode { mode: u8 },
    /// Set Column Address. Setup column start and end address.
    SetColumnAddress { column_start: u8, column_end: u8 },
    /// Set Page Address. Setup page start and end address.
    SetPageAddress { page_start: u8, page_end: u8 },
    /// Set Page Start Address for Page Addressing Mode. Set GDDRAM Page Start
    /// Address (PAGE0~PAGE7) for Page Addressing Mode using `X[2:0]`.
    SetPageStartAddress { address: u8 },

    // Hardware Configuration Commands
    /// Set Display Start Line. Set display RAM display start line register from
    /// 0-63 using `X[5:0]`.
    SetDisplayStartLine { line: u8 },
    /// Set Segment Remap.
    SetSegmentRemap { reverse: bool },
    /// Set Multiplex Ratio.
    SetMultiplexRatio { ratio: u8 },
    /// Set COM Output Scan Direction.
    SetComScanDirection { decrement: bool },
    /// Set Display Offset. Set vertical shift by COM from 0-63.
    SetDisplayOffset { vertical_shift: u8 } = 0xd3,
    /// Set COM Pins Hardware Configuration
    SetComPins { alternative: bool, enable_com: bool },

    // Timing & Driving Scheme Setting Commands.
    /// Set Display Clock Divide Ratio/Oscillator Frequency.
    SetDisplayClockDivide {
        divide_ratio: u8,
        oscillator_frequency: u8,
    },
    /// Set Pre-charge Period.
    SetPrechargePeriod { phase1: u8, phase2: u8 },
    /// Set VCOMH Deselect Level.
    SetVcomDeselect { level: u8 },
}

impl Command {
    pub fn encode(self, buffer: &mut SubSliceMut<'static, u8>) {
        let take = match self {
            Self::SetChargePump { enable } => {
                buffer[0] = 0x8D;
                buffer[1] = 0x10 | ((enable as u8) << 2);
                2
            }
            Self::SetContrast { contrast } => {
                buffer[0] = 0x81;
                buffer[1] = contrast;
                2
            }
            Self::EntireDisplayOn { ignore_ram } => {
                buffer[0] = 0xa4 | (ignore_ram as u8);
                1
            }
            Self::SetDisplayInvert { inverse } => {
                buffer[0] = 0xa6 | (inverse as u8);
                1
            }
            Self::SetDisplayOnOff { on } => {
                buffer[0] = 0xae | (on as u8);
                1
            }
            Self::ContinuousHorizontalScroll {
                left,
                page_start,
                interval,
                page_end,
            } => {
                buffer[0] = 0x26 | (left as u8);
                buffer[1] = 0;
                buffer[2] = page_start;
                buffer[3] = interval;
                buffer[4] = page_end;
                buffer[5] = 0;
                buffer[6] = 0xff;
                7
            }
            Self::ContinuousVerticalHorizontalScroll {
                left,
                page_start,
                interval,
                page_end,
                vertical_offset,
            } => {
                buffer[0] = 0x29 | (left as u8);
                buffer[1] = 0;
                buffer[2] = page_start;
                buffer[3] = interval;
                buffer[4] = page_end;
                buffer[5] = vertical_offset;
                6
            }
            Self::DeactivateScroll => {
                buffer[0] = 0x2e;
                1
            }
            Self::ActivateScroll => {
                buffer[0] = 0x2f;
                1
            }
            Self::SetVerticalScrollArea {
                rows_fixed,
                rows_scroll,
            } => {
                buffer[0] = 0xa3;
                buffer[1] = rows_fixed;
                buffer[2] = rows_scroll;
                3
            }
            Self::SetLowerColumnStartAddress { address } => {
                buffer[0] = 0x00 | (address & 0xF);
                1
            }
            Self::SetHigherColumnStartAddress { address } => {
                buffer[0] = 0x10 | ((address >> 4) & 0xF);
                1
            }
            Self::SetMemoryAddressingMode { mode } => {
                buffer[0] = 0x20;
                buffer[1] = mode;
                2
            }
            Self::SetColumnAddress {
                column_start,
                column_end,
            } => {
                buffer[0] = 0x21;
                buffer[1] = column_start;
                buffer[2] = column_end;
                3
            }
            Self::SetPageAddress {
                page_start,
                page_end,
            } => {
                buffer[0] = 0x22;
                buffer[1] = page_start;
                buffer[2] = page_end;
                3
            }
            Self::SetPageStartAddress { address } => {
                buffer[0] = 0xb0 | (address & 0x7);
                1
            }
            Self::SetDisplayStartLine { line } => {
                buffer[0] = 0x40 | (line & 0x3F);
                1
            }
            Self::SetSegmentRemap { reverse } => {
                buffer[0] = 0xa0 | (reverse as u8);
                1
            }
            Self::SetMultiplexRatio { ratio } => {
                buffer[0] = 0xa8;
                buffer[1] = ratio;
                2
            }
            Self::SetComScanDirection { decrement } => {
                buffer[0] = 0xc0 | ((decrement as u8) << 3);
                1
            }
            Self::SetDisplayOffset { vertical_shift } => {
                buffer[0] = 0xd3;
                buffer[1] = vertical_shift;
                2
            }
            Self::SetComPins {
                alternative,
                enable_com,
            } => {
                buffer[0] = 0xda;
                buffer[1] = ((alternative as u8) << 4) | ((enable_com as u8) << 5) | 0x2;
                2
            }
            Self::SetDisplayClockDivide {
                divide_ratio,
                oscillator_frequency,
            } => {
                buffer[0] = 0xd5;
                buffer[1] = ((oscillator_frequency & 0xF) << 4) | (divide_ratio & 0xf);
                2
            }
            Self::SetPrechargePeriod { phase1, phase2 } => {
                buffer[0] = 0xd9;
                buffer[1] = ((phase2 & 0xF) << 4) | (phase1 & 0xf);
                2
            }
            Self::SetVcomDeselect { level } => {
                buffer[0] = 0xdb;
                buffer[1] = (level & 0xF) << 4;
                2
            }
        };

        // Move the available region of the buffer to what is remaining after
        // this command was encoded.
        buffer.slice(take..);
    }
}

// #[derive(Copy, Clone, PartialEq)]
#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    Init,
    SimpleCommand,
    Write,
}

pub struct Ssd1306<'a, I: hil::i2c::I2CDevice> {
    i2c: &'a I,
    state: Cell<State>,
    client: OptionalCell<&'a dyn hil::screen::ScreenClient>,
    setup_client: OptionalCell<&'a dyn hil::screen::ScreenSetupClient>,
    buffer: TakeCell<'static, [u8]>,
    write_buffer: MapCell<SubSliceMut<'static, u8>>,
    enable_charge_pump: bool,
}

impl<'a, I: hil::i2c::I2CDevice> Ssd1306<'a, I> {
    pub fn new(i2c: &'a I, buffer: &'static mut [u8], enable_charge_pump: bool) -> Ssd1306<'a, I> {
        Ssd1306 {
            i2c,
            state: Cell::new(State::Idle),
            client: OptionalCell::empty(),
            setup_client: OptionalCell::empty(),
            buffer: TakeCell::new(buffer),
            write_buffer: MapCell::empty(),
            enable_charge_pump,
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
        if self.state.get() == State::Idle {
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
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, I: hil::i2c::I2CDevice> hil::screen::ScreenSetup<'a> for Ssd1306<'a, I> {
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

impl<'a, I: hil::i2c::I2CDevice> hil::screen::Screen<'a> for Ssd1306<'a, I> {
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
        let commands = [
            Command::SetPageAddress {
                page_start: (y / 8) as u8,
                page_end: ((y / 8) + (height / 8) - 1) as u8,
            },
            Command::SetColumnAddress {
                column_start: x as u8,
                column_end: (x + width - 1) as u8,
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
        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
            let mut buf_slice = SubSliceMut::new(buffer);

            // Specify this is data.
            buf_slice[0] = 0x40; // Co = 0, D/C̅ = 1

            // Move the window of the subslice after the command byte header.
            buf_slice.slice(1..);

            // Figure out how much we can send.
            let copy_len = core::cmp::min(buf_slice.len(), data.len());

            for i in 0..copy_len {
                buf_slice[i] = data[i];
            }

            let tx_len = copy_len + 1;

            self.i2c.enable();
            match self.i2c.write(buf_slice.take(), tx_len) {
                Ok(()) => {
                    self.state.set(State::Write);
                    self.write_buffer.replace(data);
                    Ok(())
                }
                Err((_e, buf)) => {
                    self.buffer.replace(buf);
                    Err(ErrorCode::INVAL)
                }
            }
        })
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

impl<I: hil::i2c::I2CDevice> hil::i2c::I2CClient for Ssd1306<'_, I> {
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

            State::Write => {
                self.state.set(State::Idle);
                self.write_buffer.take().map(|buf| {
                    self.client.map(|client| client.write_complete(buf, Ok(())));
                });
            }
            _ => {}
        }
    }
}
