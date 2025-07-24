// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Frame buffer driver for the Japan Display LPM013M126 display
//!
//! Used in Bangle.js 2 and [Jazda](https://jazda.org).
//! The driver is configured for the above devices:
//! EXTCOM inversion is driven with EXTCOMIN.
//!
//! This driver supports monochrome mode only.
//!
//! Written by Dorota <gihu.dcz@porcupinefactory.org>

use core::cell::Cell;
use core::cmp;
use kernel::debug;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::gpio::Pin;
use kernel::hil::screen::{Screen, ScreenClient, ScreenPixelFormat, ScreenRotation, ScreenSetup};
use kernel::hil::spi::{SpiMasterClient, SpiMasterDevice};
use kernel::hil::time::{Alarm, AlarmClient, ConvertTicks};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

/// 4-bit frame buffer bytes.
///
/// 176 rows, of 176 4-bit pixels and a 2-byte command header, plus a
/// trailing 2 byte transfer period
const ROWS: usize = 176;
const COLS: usize = 176;
const ROW_BYTES: usize = COLS / 2;
const LINE_LEN: usize = ROW_BYTES + 2;
pub const BUF_LEN: usize = ROWS * LINE_LEN + 2;

struct InputBuffer<'a, const PIXEL_BITS: usize> {
    data: &'a [u8],
    frame: &'a WriteFrame,
}

impl<const PIXEL_BITS: usize> InputBuffer<'_, PIXEL_BITS> {
    fn rows(&self) -> impl Iterator<Item = Row> {
        let chunk_width = if PIXEL_BITS < 8 {
            self.frame.width as usize / (8 / PIXEL_BITS)
        } else {
            self.frame.width as usize * (PIXEL_BITS / 8)
        };
        self.data.chunks(chunk_width).map(|data| Row { data })
    }
}

struct Pixel<'a> {
    data: &'a u8,
    top: bool,
}

impl Pixel<'_> {
    fn get(&self) -> u8 {
        if self.top {
            (*self.data >> 4) & 0xf
        } else {
            *self.data & 0xf
        }
    }
}

struct PixelMut<'a> {
    data: &'a Cell<u8>,
    top: bool,
}

impl PixelMut<'_> {
    fn transform<F>(&self, f: F)
    where
        F: FnOnce(&mut u8),
    {
        let mut data = if self.top {
            (self.data.get() & 0xf0) >> 4
        } else {
            self.data.get() & 0x0f
        };

        f(&mut data);

        if self.top {
            self.data.set(self.data.get() & 0x0f | ((data << 4) & 0xf0));
        } else {
            self.data.set(self.data.get() & 0xf0 | (data & 0x0f));
        }
    }
}

struct Row<'a> {
    data: &'a [u8],
}

impl<'a> Row<'a> {
    fn iter<'b>(&'b self) -> impl Iterator<Item = Pixel<'a>> {
        self.data
            .iter()
            .flat_map(|data| [Pixel { data, top: true }, Pixel { data, top: false }])
    }
}

struct RowMut<'a> {
    data: &'a [Cell<u8>],
}

impl RowMut<'_> {
    fn iter_mut(&self) -> impl Iterator<Item = PixelMut> {
        self.data
            .iter()
            .flat_map(|data| [PixelMut { data, top: true }, PixelMut { data, top: false }])
    }
}

/// Arranges frame data in a buffer
/// whose portions can be sent directly to the device.
struct FrameBuffer<'a> {
    data: SubSliceMut<'a, u8>,
}

impl<'a> FrameBuffer<'a> {
    /// Turns a regular buffer (back) into a FrameBuffer.
    /// If the buffer is fresh, and the display is initialized,
    /// this *MUST* be initialized after the call to `new`.
    fn new(mut frame_buffer: SubSliceMut<'a, u8>) -> Self {
        frame_buffer.reset();
        Self { data: frame_buffer }
    }

    /// Initialize header bytes for each line.
    fn initialize(&mut self) {
        for i in 0..ROWS {
            self.set_line_header(
                i,
                &CommandHeader {
                    mode: Mode::Input4Bit,
                    gate_line: (i + 1) as u16,
                },
            );
        }
    }

    /// Copy pixels from the buffer. The buffer may be shorter than frame.
    fn blit_rgb565(&mut self, buffer: InputBuffer<16>) {
        let frame_rows = self
            .rows()
            .skip(buffer.frame.row as usize)
            .take(buffer.frame.height as usize);
        let buf_rows = buffer.rows();

        for (frame_row, buf_row) in frame_rows.zip(buf_rows) {
            for (frame_pixel, buf_pixel) in frame_row
                .iter_mut()
                .skip(buffer.frame.column as usize)
                .zip(buf_row.data.chunks_exact(2))
            {
                let buf_pixel = [buf_pixel[0], buf_pixel[1]];
                let buf_p = u16::from_le_bytes(buf_pixel);
                frame_pixel.transform(|pixel| {
                    let red = if (buf_p >> 11) & 0b11111 >= 32 / 2 {
                        // are red five bits more than 50%?
                        0b1000
                    } else {
                        0
                    };

                    let green = if (buf_p >> 5) & 0b111111 >= 64 / 2 {
                        // green 6 bits more than 50%?
                        0b0100
                    } else {
                        0
                    };

                    let blue = if buf_p & 0b11111 >= 32 / 2 {
                        // blue five bits more than 50%?
                        0b0010
                    } else {
                        0
                    };

                    *pixel = red | green | blue;
                });
            }
        }
    }

    /// Copy pixels from the buffer. The buffer may be shorter than frame.
    fn blit_rgb332(&mut self, buffer: InputBuffer<8>) {
        let frame_rows = self
            .rows()
            .skip(buffer.frame.row as usize)
            .take(buffer.frame.height as usize);
        let buf_rows = buffer.rows();

        for (frame_row, buf_row) in frame_rows.zip(buf_rows) {
            for (frame_pixel, buf_pixel) in frame_row
                .iter_mut()
                .skip(buffer.frame.column as usize)
                .zip(buf_row.data.iter())
            {
                let buf_p: u8 = *buf_pixel;
                frame_pixel.transform(|pixel| {
                    let red = if (buf_p >> 5) & 0b111 >= 7 / 2 {
                        // are red three bits more than 50%?
                        0b1000
                    } else {
                        0
                    };

                    let green = if (buf_p >> 2) & 0b111 >= 7 / 2 {
                        // green three bits more than 50%?
                        0b0100
                    } else {
                        0
                    };

                    let blue = if buf_p & 0b11 >= 3 / 2 {
                        // blue two bits more than 50%?
                        0b0010
                    } else {
                        0
                    };

                    *pixel = red | green | blue;
                });
            }
        }
    }

    /// Copy pixels from the buffer. The buffer may be shorter than frame.
    fn blit_4bit_srgb(&mut self, buffer: InputBuffer<4>) {
        let frame_rows = self
            .rows()
            .skip(buffer.frame.row as usize)
            .take(buffer.frame.height as usize);
        let buf_rows = buffer.rows();

        for (frame_row, buf_row) in frame_rows.zip(buf_rows) {
            for (frame_pixel, buf_pixel) in frame_row
                .iter_mut()
                .skip(buffer.frame.column as usize)
                .zip(buf_row.iter())
            {
                let buf_p: u8 = buf_pixel.get();
                if buf_p & 0b1 != 0 {
                    frame_pixel.transform(|pixel| {
                        // transform from sRGB to the LPM native 4-bit format.
                        //
                        // 4-bit sRGB is encoded as `| B | G | R | s |`, where
                        // `s` is something like intensity.  We'll interpret
                        // intensity `0` to mean transparent, and intensity
                        // `1` to mean opaque.  Meanwhile LPM native 4-bit is
                        // encoded as `| R | G | B | x |`, where `x` is
                        // ignored.  So we need to swap the R & B bits, and
                        // only apply the pixel if `s` is 1.
                        *pixel = ((buf_p & 0b10) << 2) | (buf_p & 0b100) | ((buf_p & 0b1000) >> 2);
                    });
                }
            }
        }
    }

    fn set_line_header(&mut self, index: usize, header: &CommandHeader) {
        const CMD: usize = 2;
        if let Some(buf) = self.data[(LINE_LEN * index)..].first_chunk_mut::<CMD>() {
            *buf = header.encode();
        }
    }

    fn rows(&mut self) -> impl Iterator<Item = RowMut> {
        self.data.as_slice().chunks_mut(LINE_LEN).map_while(|c| {
            c.get_mut(2..).map(|data| RowMut {
                data: Cell::from_mut(data).as_slice_of_cells(),
            })
        })
    }
}

/// Modes are 6-bit, network order.
/// They use a tree-ish encoding, so only the ones in use are listed here.
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Mode {
    /// Clear memory
    /// bits: 0 Function, X, 1 Clear, 0 Blink off, X, X
    AllClear = 0b001000,
    /// Input 1-bit data
    /// bits: 1 No function, X, 0 Data Update, 01 1-bit, X
    Input1Bit = 0b100_01_0,
    Input4Bit = 0b100100,
    NoUpdate = 0b101000,
}

/// Command header is composed of a 6-bit mode and 10 bits of address,
/// network bit order.
struct CommandHeader {
    mode: Mode,
    gate_line: u16,
}

impl CommandHeader {
    /// Formats header for transfer
    fn encode(&self) -> [u8; 2] {
        ((self.gate_line & 0b1111111111) | ((self.mode as u16) << 10)).to_be_bytes()
    }
}

/// Area of the screen to which data is written
#[derive(Debug, Copy, Clone)]
struct WriteFrame {
    row: u16,
    column: u16,
    width: u16,
    height: u16,
}

/// Internal state of the driver.
/// Each state can lead to the next one in order of appearance.
#[derive(Debug, Copy, Clone)]
enum State {
    /// Data structures not ready, call `setup`
    Uninitialized,

    /// Display hardware is off, uninitialized.
    Off,
    InitializingPixelMemory,
    /// COM polarity and internal latch circuits
    InitializingRest,

    // Normal operation
    Idle,
    AllClearing,
    Writing,

    /// This driver is buggy. Turning off and on will try to recover it.
    Bug,
}

#[derive(Debug)]
pub enum InitError {
    BufferTooSmall,
}

pub struct Lpm013m126<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> {
    spi: &'a S,
    extcomin: &'a P,
    disp: &'a P,

    state: Cell<State>,

    pixel_format: Cell<ScreenPixelFormat>,
    frame: Cell<WriteFrame>,

    /// Fields responsible for sending callbacks
    /// for actions completed in software.
    ready_callback: DeferredCall,
    ready_callback_handler: ReadyCallbackHandler<'a, A, P, S>,
    command_complete_callback: DeferredCall,
    command_complete_callback_handler: CommandCompleteCallbackHandler<'a, A, P, S>,

    /// The HIL requires updates to arbitrary rectangles.
    /// The display supports only updating entire rows,
    /// so edges need to be cached.
    frame_buffer: OptionalCell<FrameBuffer<'static>>,

    client: OptionalCell<&'a dyn ScreenClient>,
    /// Buffer for incoming pixel data, coming from the client.
    /// It's not submitted directly anywhere.
    buffer: TakeCell<'static, [u8]>,

    /// Needed for init and to flip the EXTCOMIN pin at regular intervals
    alarm: &'a A,
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> Lpm013m126<'a, A, P, S>
where
    Self: 'static,
{
    pub fn new(
        spi: &'a S,
        extcomin: &'a P,
        disp: &'a P,
        alarm: &'a A,
        frame_buffer: &'static mut [u8; BUF_LEN],
    ) -> Result<Self, InitError> {
        Ok(Self {
            spi,
            alarm,
            disp,
            extcomin,
            ready_callback: DeferredCall::new(),
            ready_callback_handler: ReadyCallbackHandler::new(),
            command_complete_callback: DeferredCall::new(),
            command_complete_callback_handler: CommandCompleteCallbackHandler::new(),
            frame_buffer: OptionalCell::new(FrameBuffer::new((frame_buffer as &mut [u8]).into())),
            pixel_format: Cell::new(ScreenPixelFormat::RGB_565),
            buffer: TakeCell::empty(),
            client: OptionalCell::empty(),
            state: Cell::new(State::Uninitialized),
            frame: Cell::new(WriteFrame {
                row: 0,
                column: 0,
                width: COLS as u16,
                height: ROWS as u16,
            }),
        })
    }

    /// Set up internal data structures.
    /// Does not touch the hardware.
    /// Idempotent.
    pub fn setup(&'static self) -> Result<(), ErrorCode> {
        // Needed this way to avoid exposing accessors to deferred callers.
        // That would be unnecessary, no external data is needed.
        // At the same time, self must be static for client registration.
        match self.state.get() {
            State::Uninitialized => {
                self.ready_callback_handler.lpm.set(self);
                self.ready_callback.register(&self.ready_callback_handler);
                self.command_complete_callback_handler.lpm.set(self);
                self.command_complete_callback
                    .register(&self.command_complete_callback_handler);

                self.state.set(State::Off);
                Ok(())
            }
            _ => Err(ErrorCode::ALREADY),
        }
    }

    fn initialize(&self) -> Result<(), ErrorCode> {
        match self.state.get() {
            State::Off | State::Bug => {
                // Even if we took Pin type that implements Output,
                // it's still possible that it is *not configured as a output*
                // at the moment.
                // To ensure outputness, output must be configured at runtime,
                // even though this eliminates pins
                // which don't implement Configure due to being
                // simple, unconfigurable outputs.
                self.extcomin.make_output();
                self.extcomin.clear();
                self.disp.make_output();
                self.disp.clear();

                match self.frame_buffer.take() {
                    None => Err(ErrorCode::NOMEM),
                    Some(mut frame_buffer) => {
                        // Cheating a little:
                        // the frame buffer does not yet contain pixels,
                        // so use its beginning to send the clear command.
                        frame_buffer.set_line_header(
                            0,
                            &CommandHeader {
                                mode: Mode::AllClear,
                                gate_line: 0,
                            },
                        );
                        let mut l = frame_buffer.data;
                        l.slice(0..2);
                        let res = self.spi.read_write_bytes(l, None);

                        let (res, new_state) = match res {
                            Ok(()) => (Ok(()), State::InitializingPixelMemory),
                            Err((e, buf, _)) => {
                                self.frame_buffer.replace(FrameBuffer::new(buf));
                                (Err(e), State::Bug)
                            }
                        };
                        self.state.set(new_state);
                        res
                    }
                }
            }
            _ => Err(ErrorCode::ALREADY),
        }
    }

    fn uninitialize(&self) -> Result<(), ErrorCode> {
        match self.state.get() {
            State::Off => Err(ErrorCode::ALREADY),
            _ => {
                // TODO: investigate clearing pixels asynchronously,
                // like the datasheet asks.
                // It seems to turn off fine without clearing, but
                // perhaps the state of the buffer affects power draw when off.

                // The following stops extcomin timer.
                self.alarm.disarm()?;
                self.disp.clear();
                self.state.set(State::Off);

                self.ready_callback.set();
                Ok(())
            }
        }
    }

    fn arm_alarm(&self) {
        // Datasheet says 2Hz or more often flipping is required
        // for transmissive mode.
        let delay = self.alarm.ticks_from_ms(100);
        self.alarm.set_alarm(self.alarm.now(), delay);
    }

    fn handle_ready_callback(&self) {
        self.client.map(|client| client.screen_is_ready());
    }

    fn handle_command_complete_callback(&self) {
        // Thankfully, this is the only command that results in the callback,
        // so there's no danger that this will get attributed
        // to a command that's not finished yet.
        self.client.map(|client| client.command_complete(Ok(())));
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> Screen<'a> for Lpm013m126<'a, A, P, S>
where
    Self: 'static,
{
    fn get_resolution(&self) -> (usize, usize) {
        (ROWS, COLS)
    }

    fn get_pixel_format(&self) -> ScreenPixelFormat {
        self.pixel_format.get()
    }

    fn get_rotation(&self) -> ScreenRotation {
        ScreenRotation::Normal
    }

    fn set_write_frame(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<(), ErrorCode> {
        let (columns, rows) = self.get_resolution();
        if y >= rows || y + height > rows || x >= columns || x + width > columns {
            //return Err(ErrorCode::INVAL);
        }

        let frame = WriteFrame {
            row: y as u16,
            column: x as u16,
            width: width as u16,
            height: height as u16,
        };
        self.frame.set(frame);

        self.command_complete_callback.set();

        Ok(())
    }

    fn write(
        &self,
        data: SubSliceMut<'static, u8>,
        _continue_write: bool,
    ) -> Result<(), ErrorCode> {
        let len = data.len();
        let buffer = data.take();

        let ret = match self.state.get() {
            State::Uninitialized | State::Off => Err(ErrorCode::OFF),
            State::InitializingPixelMemory | State::InitializingRest => Err(ErrorCode::BUSY),
            State::Idle => {
                self.frame_buffer
                    .take()
                    .map_or(Err(ErrorCode::NOMEM), |mut frame_buffer| {
                        match self.pixel_format.get() {
                            ScreenPixelFormat::RGB_332 => {
                                frame_buffer.blit_rgb332(InputBuffer {
                                    data: &buffer[..cmp::min(buffer.len(), len)],
                                    frame: &self.frame.get(),
                                });
                            }
                            ScreenPixelFormat::RGB_565 => {
                                frame_buffer.blit_rgb565(InputBuffer {
                                    data: &buffer[..cmp::min(buffer.len(), len)],
                                    frame: &self.frame.get(),
                                });
                            }
                            _ => frame_buffer.blit_4bit_srgb(InputBuffer {
                                data: &buffer[..cmp::min(buffer.len(), len)],
                                frame: &self.frame.get(),
                            }),
                        }

                        frame_buffer.set_line_header(
                            0,
                            &CommandHeader {
                                mode: Mode::NoUpdate,
                                gate_line: 0,
                            },
                        );
                        let mut l = frame_buffer.data;
                        l.slice(0..2);
                        let sent = self.spi.read_write_bytes(l, None);

                        let (ret, new_state) = match sent {
                            Ok(()) => (Ok(()), State::AllClearing),
                            Err((e, buf, _)) => {
                                self.frame_buffer.replace(FrameBuffer::new(buf));
                                (Err(e), State::Idle)
                            }
                        };
                        self.state.set(new_state);
                        ret
                    })
            }
            State::AllClearing | State::Writing => Err(ErrorCode::BUSY),
            State::Bug => Err(ErrorCode::FAIL),
        };

        self.buffer.replace(buffer);

        ret
    }

    fn set_client(&self, client: &'a dyn ScreenClient) {
        self.client.set(client);
    }

    fn set_power(&self, enable: bool) -> Result<(), ErrorCode> {
        let ret = if enable {
            self.initialize()
        } else {
            self.uninitialize()
        };

        // If the device is in the desired state by now,
        // then a callback needs to be sent manually.
        if let Err(ErrorCode::ALREADY) = ret {
            self.ready_callback.set();
            Ok(())
        } else {
            ret
        }
    }

    fn set_brightness(&self, _brightness: u16) -> Result<(), ErrorCode> {
        // TODO: add LED PWM
        Err(ErrorCode::NOSUPPORT)
    }

    fn set_invert(&self, _inverted: bool) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> ScreenSetup<'a> for Lpm013m126<'a, A, P, S> {
    fn set_client(&self, _client: &'a dyn kernel::hil::screen::ScreenSetupClient) {
        todo!()
    }

    fn set_resolution(&self, resolution: (usize, usize)) -> Result<(), ErrorCode> {
        if resolution == (ROWS, COLS) {
            Ok(())
        } else {
            Err(ErrorCode::NOSUPPORT)
        }
    }

    fn set_pixel_format(&self, format: ScreenPixelFormat) -> Result<(), ErrorCode> {
        match format {
            ScreenPixelFormat::RGB_4BIT
            | ScreenPixelFormat::RGB_332
            | ScreenPixelFormat::RGB_565 => {
                self.pixel_format.set(format);
                Ok(())
            }
            _ => Err(ErrorCode::NOSUPPORT),
        }
    }

    fn set_rotation(&self, _rotation: ScreenRotation) -> Result<(), ErrorCode> {
        todo!()
    }

    fn get_num_supported_resolutions(&self) -> usize {
        1
    }

    fn get_supported_resolution(&self, index: usize) -> Option<(usize, usize)> {
        match index {
            0 => Some((ROWS, COLS)),
            _ => None,
        }
    }

    fn get_num_supported_pixel_formats(&self) -> usize {
        3
    }

    fn get_supported_pixel_format(&self, index: usize) -> Option<ScreenPixelFormat> {
        match index {
            0 => Some(ScreenPixelFormat::RGB_4BIT),
            1 => Some(ScreenPixelFormat::RGB_332),
            2 => Some(ScreenPixelFormat::RGB_565),
            _ => None,
        }
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> AlarmClient for Lpm013m126<'a, A, P, S>
where
    Self: 'static,
{
    fn alarm(&self) {
        match self.state.get() {
            State::InitializingRest => {
                // Better flip it once too many than go out of spec
                // by stretching the flip period.
                self.extcomin.set();
                self.disp.set();
                self.arm_alarm();
                let new_state = self.frame_buffer.take().map_or_else(
                    || {
                        debug!(
                            "LPM013M126 driver lost its frame buffer in state {:?}",
                            self.state.get()
                        );
                        State::Bug
                    },
                    |mut buffer| {
                        buffer.initialize();
                        self.frame_buffer.replace(buffer);
                        State::Idle
                    },
                );

                self.state.set(new_state);

                if let State::Idle = new_state {
                    self.client.map(|client| client.screen_is_ready());
                }
            }
            _ => {
                self.extcomin.toggle();
            }
        }
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> SpiMasterClient for Lpm013m126<'a, A, P, S> {
    fn read_write_done(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        _read_buffer: Option<SubSliceMut<'static, u8>>,
        status: Result<usize, ErrorCode>,
    ) {
        self.frame_buffer.replace(FrameBuffer::new(write_buffer));
        self.state.set(match self.state.get() {
            State::InitializingPixelMemory => {
                // Rather than initialize them separately, wait longer and do both
                // for 2 reasons:
                // 1. the upper limit of waiting is only specified for both,
                // 2. and state flipping code is annoying and bug-friendly.
                let delay = self.alarm.ticks_from_us(150);
                self.alarm.set_alarm(self.alarm.now(), delay);
                State::InitializingRest
            }
            State::AllClearing => {
                if let Some(mut fb) = self.frame_buffer.take() {
                    fb.set_line_header(
                        0,
                        &CommandHeader {
                            mode: Mode::Input4Bit,
                            gate_line: 1,
                        },
                    );
                    let mut send_buf = fb.data;

                    let first_row = cmp::min(ROWS as u16, self.frame.get().row);
                    let offset = first_row as usize * LINE_LEN;
                    let len = cmp::min(ROWS as u16 - first_row, self.frame.get().height) as usize
                        * LINE_LEN;
                    send_buf.slice(offset..(offset + len + 2));

                    let _ = self.spi.read_write_bytes(send_buf, None);
                }
                State::Writing
            }
            State::Writing => {
                if let Some(mut fb) = self.frame_buffer.take() {
                    fb.initialize();
                    self.frame_buffer.set(fb);
                }
                State::Idle
            }
            // can't get more buggy than buggy
            other => {
                debug!(
                    "LPM013M126 received unexpected SPI complete in state {:?}",
                    other
                );
                State::Bug
            }
        });

        if let State::Idle = self.state.get() {
            // Device frame buffer is now up to date, return pixel buffer to client.
            self.client.map(|client| {
                self.buffer.take().map(|buf| {
                    let data = SubSliceMut::new(buf);
                    client.write_complete(data, status.map(|_| ()))
                })
            });
        }
    }
}

// DeferredCall requires a unique client for each DeferredCall so that different callbacks
// can be distinguished.
struct ReadyCallbackHandler<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> {
    lpm: OptionalCell<&'a Lpm013m126<'a, A, P, S>>,
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> ReadyCallbackHandler<'a, A, P, S> {
    fn new() -> Self {
        Self {
            lpm: OptionalCell::empty(),
        }
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> DeferredCallClient
    for ReadyCallbackHandler<'a, A, P, S>
where
    Self: 'static,
{
    fn handle_deferred_call(&self) {
        self.lpm.map(|l| l.handle_ready_callback());
    }

    fn register(&'static self) {
        self.lpm.map(|l| l.ready_callback.register(self));
    }
}

struct CommandCompleteCallbackHandler<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> {
    lpm: OptionalCell<&'a Lpm013m126<'a, A, P, S>>,
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> CommandCompleteCallbackHandler<'a, A, P, S> {
    fn new() -> Self {
        Self {
            lpm: OptionalCell::empty(),
        }
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice<'a>> DeferredCallClient
    for CommandCompleteCallbackHandler<'a, A, P, S>
where
    Self: 'static,
{
    fn handle_deferred_call(&self) {
        self.lpm.map(|l| l.handle_command_complete_callback());
    }

    fn register(&'static self) {
        self.lpm.map(|l| l.command_complete_callback.register(self));
    }
}
