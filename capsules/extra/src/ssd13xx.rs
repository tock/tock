// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! SSD1306/SSD1309/SSD1315 OLED Screen
//!
//! The SSD1306, SSD1309, and SSD1315 share their command set and 128x64 display
//! RAM. The exact chip is selected through the [`Variant`] generic type
//! parameter ([`Ssd1306`], [`Ssd1309`], or [`Ssd1315`]). Select the panel
//! resolution with the `WIDTH` and `HEIGHT` const generics (128x64 by default).
//! Panels narrower than 128 columns are assumed to be centered in the
//! controller's columns.
//!
//! Both controllers can be attached over I2C ([`I2CTransport`]) or 4-wire SPI
//! with a data/command pin ([`SpiTransport`]).

use core::cell::Cell;
use kernel::ErrorCode;
use kernel::hil;
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;

pub const BUFFER_SIZE: usize = 1032;

/// Number of columns in the controller's display RAM.
const CONTROLLER_WIDTH: usize = 128;
/// Number of rows (COM lines) of the controller.
const CONTROLLER_HEIGHT: usize = 64;

/// Settings that differ between the controllers this driver supports.
pub trait Variant {
    /// Whether the controller has an internal charge pump (command `0x8D`).
    const HAS_CHARGE_PUMP: bool;

    /// Controller-specific commands sent during initialization, before the
    /// display is turned on.
    const INIT_COMMANDS: &'static [Command];
}

/// The SSD1306 controller.
pub enum Ssd1306 {}

impl Variant for Ssd1306 {
    const HAS_CHARGE_PUMP: bool = true;
    const INIT_COMMANDS: &'static [Command] = &[
        Command::SetPrechargePeriod {
            phase1: 0x1,
            phase2: 0xf,
        },
        Command::SetVcomDeselect { level: 2 },
    ];
}

/// The SSD1315 controller, which is compatible with the SSD1306.
pub type Ssd1315 = Ssd1306;

/// The SSD1309 controller.
///
/// The SSD1309 has no charge pump (the panel voltage is supplied externally)
/// and encodes the VCOMH deselect level differently from the SSD1306. It keeps
/// its reset defaults for the pre-charge period (`0x22`) and VCOMH level
/// (`0x34`, ~0.78 x VCC).
pub enum Ssd1309 {}

impl Variant for Ssd1309 {
    const HAS_CHARGE_PUMP: bool = false;
    const INIT_COMMANDS: &'static [Command] = &[];
}

/// How the panel is wired to the controller's segment (column) and COM (row)
/// lines. This depends on the display module, and decides the orientation of
/// the image.
///
/// The fields are named after the controller commands they configure.
#[derive(Clone, Copy)]
pub struct PanelConfig {
    /// Set Segment Re-map (`0xA0`/`0xA1`).
    ///
    /// `true` maps column 0 to SEG127, mirroring the image horizontally.
    pub segment_remap: bool,
    /// Set COM Output Scan Direction (`0xC0`/`0xC8`).
    ///
    /// `true` scans from COM[N-1] to COM0, mirroring the image vertically.
    pub com_scan_decrement: bool,
    /// Set COM Pins Hardware Configuration (`0xDA`).
    ///
    /// `true` means alternative (interleaved) instead of sequential COM pin
    /// configuration. Panels with 64 rows typically use the alternative
    /// configuration, panels with 32 rows the sequential one.
    pub com_pins_alternative: bool,
    /// Set COM Pins Hardware Configuration (`0xDA`).
    ///
    /// `true` means enabling COM left/right remap.
    pub com_left_right_remap: bool,
}

impl Default for PanelConfig {
    /// The configuration of common 128x64 modules.
    fn default() -> Self {
        PanelConfig {
            segment_remap: true,
            com_scan_decrement: true,
            com_pins_alternative: true,
            com_left_right_remap: false,
        }
    }
}

/// Whether bytes sent to the controller are commands or display data.
#[derive(Clone, Copy, PartialEq)]
pub enum DataKind {
    Command,
    Data,
}

/// Bus that connects the controller.
pub trait Transport<'a> {
    fn set_client(&self, client: &'a dyn TransportClient);

    /// Send `buffer[1..len]` to the controller as `kind` bytes.
    ///
    /// `buffer[0]` is reserved for the transport (e.g. the I2C control byte).
    /// On success, [`TransportClient::send_done`] returns the buffer. `len`
    /// includes this byte.
    fn send(
        &self,
        kind: DataKind,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;
}

pub trait TransportClient {
    fn send_done(&self, buffer: &'static mut [u8], result: Result<(), ErrorCode>);
}

/// I2C transport for SSD13xx screens.
///
/// Each transfer starts with a control byte that marks the following bytes as
/// commands or data.
pub struct I2CTransport<'a, I: hil::i2c::I2CDevice<'a>> {
    i2c: &'a I,
    client: OptionalCell<&'a dyn TransportClient>,
}

impl<'a, I: hil::i2c::I2CDevice<'a>> I2CTransport<'a, I> {
    pub fn new(i2c: &'a I) -> Self {
        Self {
            i2c,
            client: OptionalCell::empty(),
        }
    }
}

impl<'a, I: hil::i2c::I2CDevice<'a>> Transport<'a> for I2CTransport<'a, I> {
    fn set_client(&self, client: &'a dyn TransportClient) {
        self.client.set(client);
    }

    fn send(
        &self,
        kind: DataKind,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        buffer[0] = match kind {
            DataKind::Command => 0, // Co = 0, D/C̅ = 0
            DataKind::Data => 0x40, // Co = 0, D/C̅ = 1
        };

        self.i2c.enable();
        self.i2c.write(buffer, len).map_err(|(e, buf)| {
            self.i2c.disable();
            (e.into(), buf)
        })
    }
}

impl<'a, I: hil::i2c::I2CDevice<'a>> hil::i2c::I2CClient for I2CTransport<'a, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), hil::i2c::Error>) {
        self.i2c.disable();
        self.client
            .map(|client| client.send_done(buffer, status.map_err(Into::into)));
    }
}

/// 4-wire SPI transport: the D/C̅ pin marks the transferred bytes as commands
/// (low) or data (high).
///
/// The transport uses the SPI controller directly, so the controller cannot
/// be shared with other devices. Its chip select, clock rate, and mode must be
/// configured before use (SPI mode 0).
pub struct SpiTransport<'a, S: hil::spi::SpiMaster<'a>, P: hil::gpio::Output> {
    spi: &'a S,
    dc: &'a P,
    client: OptionalCell<&'a dyn TransportClient>,
}

impl<'a, S: hil::spi::SpiMaster<'a>, P: hil::gpio::Output> SpiTransport<'a, S, P> {
    pub fn new(spi: &'a S, dc: &'a P) -> Self {
        Self {
            spi,
            dc,
            client: OptionalCell::empty(),
        }
    }
}

impl<'a, S: hil::spi::SpiMaster<'a>, P: hil::gpio::Output> Transport<'a>
    for SpiTransport<'a, S, P>
{
    fn set_client(&self, client: &'a dyn TransportClient) {
        self.client.set(client);
    }

    fn send(
        &self,
        kind: DataKind,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        match kind {
            DataKind::Command => self.dc.clear(),
            DataKind::Data => self.dc.set(),
        }

        let mut buf_slice = SubSliceMut::new(buffer);
        buf_slice.slice(1..len);
        self.spi
            .read_write_bytes(buf_slice, None)
            .map_err(|(e, buf_slice, _)| (e, buf_slice.take()))
    }
}

impl<'a, S: hil::spi::SpiMaster<'a>, P: hil::gpio::Output> hil::spi::SpiMasterClient
    for SpiTransport<'a, S, P>
{
    fn read_write_done(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        _read_buffer: Option<SubSliceMut<'static, u8>>,
        status: Result<usize, ErrorCode>,
    ) {
        self.client
            .map(|client| client.send_done(write_buffer.take(), status.map(|_| ())));
    }
}

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

pub struct Ssd13xx<
    'a,
    V: Variant,
    T: Transport<'a>,
    const WIDTH: usize = CONTROLLER_WIDTH,
    const HEIGHT: usize = CONTROLLER_HEIGHT,
> {
    transport: &'a T,
    state: Cell<State>,
    client: OptionalCell<&'a dyn hil::screen::ScreenClient>,
    setup_client: OptionalCell<&'a dyn hil::screen::ScreenSetupClient>,
    buffer: TakeCell<'static, [u8]>,
    write_buffer: MapCell<SubSliceMut<'static, u8>>,
    enable_charge_pump: bool,
    panel: PanelConfig,
    _variant: core::marker::PhantomData<V>,
}

impl<'a, V: Variant, T: Transport<'a>, const WIDTH: usize, const HEIGHT: usize>
    Ssd13xx<'a, V, T, WIDTH, HEIGHT>
{
    /// Column of the controller's display RAM where the panel starts.
    const COLUMN_OFFSET: usize = (CONTROLLER_WIDTH - WIDTH) / 2;

    /// Rejects unsupported resolutions at compile time.
    const VALID_RESOLUTION: () = assert!(
        WIDTH > 0
            && WIDTH <= CONTROLLER_WIDTH
            && HEIGHT > 0
            && HEIGHT <= CONTROLLER_HEIGHT
            && HEIGHT.is_multiple_of(8)
    );

    /// Create the screen driver.
    ///
    /// `enable_charge_pump` is ignored for controllers without a charge pump.
    pub fn new(
        transport: &'a T,
        buffer: &'static mut [u8],
        enable_charge_pump: bool,
        panel: PanelConfig,
    ) -> Self {
        let () = Self::VALID_RESOLUTION;

        Ssd13xx {
            transport,
            state: Cell::new(State::Idle),
            client: OptionalCell::empty(),
            setup_client: OptionalCell::empty(),
            buffer: TakeCell::new(buffer),
            write_buffer: MapCell::empty(),
            enable_charge_pump,
            panel,
            _variant: core::marker::PhantomData,
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
            Command::SetMemoryAddressingMode { mode: 0 }, //horizontal
            Command::SetSegmentRemap {
                reverse: self.panel.segment_remap,
            },
            Command::SetComScanDirection {
                decrement: self.panel.com_scan_decrement,
            },
            Command::SetComPins {
                alternative: self.panel.com_pins_alternative,
                enable_com: self.panel.com_left_right_remap,
            },
            Command::SetContrast { contrast: 0xcf },
            Command::EntireDisplayOn { ignore_ram: false },
            Command::SetDisplayInvert { inverse: false },
            Command::DeactivateScroll,
        ];
        let charge_pump = Command::SetChargePump {
            enable: self.enable_charge_pump,
        };

        let sequence = commands
            .iter()
            .chain(V::HAS_CHARGE_PUMP.then_some(&charge_pump))
            .chain(V::INIT_COMMANDS)
            .chain(&[Command::SetDisplayOnOff { on: true }]);

        match self.send_sequence(sequence) {
            Ok(()) => {
                self.state.set(State::Init);
            }
            Err(_e) => {}
        }
    }

    fn send_sequence<'c>(
        &self,
        sequence: impl IntoIterator<Item = &'c Command>,
    ) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
                let mut buf_slice = SubSliceMut::new(buffer);

                // Move the window of the subslice after the header byte
                // reserved for the transport.
                buf_slice.slice(1..);

                for cmd in sequence {
                    cmd.encode(&mut buf_slice);
                }

                // We need the amount of data that has been sliced away
                // at the start of the subslice.
                let remaining_len = buf_slice.len();
                buf_slice.reset();
                let tx_len = buf_slice.len() - remaining_len;

                match self
                    .transport
                    .send(DataKind::Command, buf_slice.take(), tx_len)
                {
                    Ok(()) => Ok(()),
                    Err((e, buf)) => {
                        self.buffer.replace(buf);
                        Err(e)
                    }
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, V: Variant, T: Transport<'a>, const WIDTH: usize, const HEIGHT: usize>
    hil::screen::ScreenSetup<'a> for Ssd13xx<'a, V, T, WIDTH, HEIGHT>
{
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

impl<'a, V: Variant, T: Transport<'a>, const WIDTH: usize, const HEIGHT: usize>
    hil::screen::Screen<'a> for Ssd13xx<'a, V, T, WIDTH, HEIGHT>
{
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
                column_start: (Self::COLUMN_OFFSET + x) as u8,
                column_end: (Self::COLUMN_OFFSET + x + width - 1) as u8,
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

            // Move the window of the subslice after the header byte reserved
            // for the transport.
            buf_slice.slice(1..);

            // Figure out how much we can send.
            let copy_len = core::cmp::min(buf_slice.len(), data.len());

            for i in 0..copy_len {
                buf_slice[i] = data[i];
            }

            let tx_len = copy_len + 1;

            match self
                .transport
                .send(DataKind::Data, buf_slice.take(), tx_len)
            {
                Ok(()) => {
                    self.state.set(State::Write);
                    self.write_buffer.replace(data);
                    Ok(())
                }
                Err((e, buf)) => {
                    self.buffer.replace(buf);
                    Err(e)
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

impl<'a, V: Variant, T: Transport<'a>, const WIDTH: usize, const HEIGHT: usize> TransportClient
    for Ssd13xx<'a, V, T, WIDTH, HEIGHT>
{
    fn send_done(&self, buffer: &'static mut [u8], result: Result<(), ErrorCode>) {
        self.buffer.replace(buffer);

        match self.state.get() {
            State::Init => {
                self.state.set(State::Idle);
                self.client.map(|client| client.screen_is_ready());
            }

            State::SimpleCommand => {
                self.state.set(State::Idle);
                self.client.map(|client| client.command_complete(result));
            }

            State::Write => {
                self.state.set(State::Idle);
                self.write_buffer.take().map(|buf| {
                    self.client.map(|client| client.write_complete(buf, result));
                });
            }
            _ => {}
        }
    }
}
