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
use kernel::hil::screen::{Screen, ScreenClient, ScreenPixelFormat, ScreenRotation};
use kernel::hil::spi::{SpiMasterClient, SpiMasterDevice};
use kernel::hil::time::{Alarm, AlarmClient, ConvertTicks};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

/// Monochrome frame buffer bytes.
/// 176 × 176 bits = 3872 bytes.
///
/// 2 bytes for the start of each row (command header),
/// plus 2 bytes of data transfer period at the end
///
/// 176 * 2 + 2 = 354 bytes.
pub const BUF_LEN: usize = 3872 + 354;

/// Best Tock can do, sadly.
/// Would be better to have it offset.
type SubmitBuffer<'a> = &'a mut [u8];

/// Arranges frame data in a buffer
/// whose portions can be sent directly to the device.
struct FrameBuffer<'a> {
    data: &'a mut [u8],
}

impl<'a> FrameBuffer<'a> {
    /// Turns a regular buffer (back) into a FrameBuffer.
    /// If the buffer is fresh, and the display is initialized,
    /// this *MUST* be initialized after the call to `new`.
    fn new(frame_buffer: &'a mut [u8]) -> Self {
        Self { data: frame_buffer }
    }

    /// Initialize header bytes for each line.
    fn initialize(&mut self) {
        for i in 0..176 {
            let line = self.get_line_mut(i);
            let bytes = CommandHeader {
                mode: Mode::Input1Bit,
                gate_line: i,
            }
            .encode();
            line[..2].copy_from_slice(&bytes);
        }
    }

    /// Copy pixels from the buffer. The buffer may be shorter than frame.
    fn blit(&mut self, buffer: &[u8], frame: &WriteFrame) {
        if frame.column % 8 != 0 {
            // Can't be arsed to bit shift pixels…
            panic!("Horizontal offset not supported");
        }
        let rows = (frame.row)..(frame.row + frame.height);
        // There are 8 pixels in each row per byte.
        let sources = buffer.chunks(frame.width as usize / 8);
        for (i, source) in rows.zip(sources) {
            let row = self.get_row_mut(i);
            row[(frame.column as usize / 8)..][..(source.len())].copy_from_slice(source);
        }
    }

    /// Gets an entire raw line, ready to send.
    fn get_line_mut(&mut self, index: u16) -> &mut [u8] {
        const CMD: usize = 2;
        const TRANSFER_PERIOD: usize = 2;
        let line_bytes = CMD + 176 / 8;
        &mut self.data[(line_bytes * index as usize)..][..line_bytes + TRANSFER_PERIOD]
    }

    /// Gets pixel data.
    fn get_row_mut(&mut self, index: u16) -> &mut [u8] {
        let line_bytes = 176 / 8 + 2;
        &mut self.data[(line_bytes * index as usize + 2)..][..(176 / 8)]
    }

    /// Transform into a view of raw data for submitting to the DMA driver
    fn with_raw_rows(
        frame_buffer: FrameBuffer<'static>,
        _start: u16,
        _end: u16,
    ) -> SubmitBuffer<'static> {
        // HILs typically can't use offsets :/
        // Best we can do is limit length (TODO)
        frame_buffer.data
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
        (self.gate_line | ((self.mode as u16) << 10)).to_be_bytes()
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
    Idle(WriteFrame),
    Writing(WriteFrame),

    /// This driver is buggy. Turning off and on will try to recover it.
    Bug,
}

#[derive(Debug)]
pub enum InitError {
    BufferTooSmall,
}

pub struct Lpm013m126<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> {
    spi: &'a S,
    extcomin: &'a P,
    disp: &'a P,

    state: Cell<State>,

    /// Fields responsible for sending callbacks
    /// for actions completed in software.
    ready_callback: DeferredCall,
    ready_callback_handler: ReadyCallbackHandler<'a, A, P, S>,
    command_complete_callback: DeferredCall,
    command_complete_callback_handler: CommandCompleteCallbackHandler<'a, A, P, S>,
    write_complete_callback: DeferredCall,
    write_complete_callback_handler: WriteCompleteCallbackHandler<'a, A, P, S>,
    /// Holds the pending call parameter
    write_complete_pending_call: OptionalCell<Result<(), ErrorCode>>,

    /// The HIL requires updates to arbitrary rectangles.
    /// The display supports only updating entire rows,
    /// so edges need to be cached.
    frame_buffer: OptionalCell<FrameBuffer<'static>>,

    client: OptionalCell<&'static dyn ScreenClient>,
    /// Buffer for incoming pixel data, coming from the client.
    /// It's not submitted directly anywhere.
    buffer: TakeCell<'static, [u8]>,

    /// Needed for init and to flip the EXTCOMIN pin at regular intervals
    alarm: &'a A,
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> Lpm013m126<'a, A, P, S>
where
    Self: 'static,
{
    pub fn new(
        spi: &'a S,
        extcomin: &'a P,
        disp: &'a P,
        alarm: &'a A,
        frame_buffer: &'static mut [u8],
    ) -> Result<Self, InitError> {
        if frame_buffer.len() < BUF_LEN {
            Err(InitError::BufferTooSmall)
        } else {
            Ok(Self {
                spi,
                alarm,
                disp,
                extcomin,
                ready_callback: DeferredCall::new(),
                ready_callback_handler: ReadyCallbackHandler::new(),
                command_complete_callback: DeferredCall::new(),
                command_complete_callback_handler: CommandCompleteCallbackHandler::new(),
                write_complete_callback: DeferredCall::new(),
                write_complete_callback_handler: WriteCompleteCallbackHandler::new(),
                write_complete_pending_call: OptionalCell::empty(),
                frame_buffer: OptionalCell::new(FrameBuffer::new(frame_buffer)),
                buffer: TakeCell::empty(),
                client: OptionalCell::empty(),
                state: Cell::new(State::Uninitialized),
            })
        }
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
                self.write_complete_callback_handler.lpm.set(self);
                self.write_complete_callback
                    .register(&self.write_complete_callback_handler);

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
                        let buf = &mut frame_buffer.get_line_mut(0)[..2];
                        buf.copy_from_slice(
                            &CommandHeader {
                                mode: Mode::AllClear,
                                gate_line: 0,
                            }
                            .encode(),
                        );
                        let l = FrameBuffer::with_raw_rows(frame_buffer, 0, 1);
                        let res = self.spi.read_write_bytes(
                            l, //FrameBuffer::with_raw_rows(frame_buffer, 0, 1),
                            None, 2,
                        );

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

    fn call_write_complete(&self, ret: Result<(), ErrorCode>) {
        self.write_complete_callback.set();
        self.write_complete_pending_call.set(ret);
    }

    fn arm_alarm(&self) {
        // Datasheet says 2Hz or more often flipping is required
        // for transmissive mode.
        let delay = self.alarm.ticks_from_ms(500);
        self.alarm.set_alarm(self.alarm.now(), delay);
    }

    fn handle_ready_callback(&self) {
        self.client.map(|client| client.screen_is_ready());
    }

    fn handle_write_complete_callback(&self) {
        self.client.map(|client| {
            self.write_complete_pending_call.map(|pend| {
                self.buffer
                    .take()
                    .map(|buffer| client.write_complete(buffer, *pend));
            });
            self.write_complete_pending_call.take();
        });
    }

    fn handle_command_complete_callback(&self) {
        // Thankfully, this is the only command that results in the callback,
        // so there's no danger that this will get attributed
        // to a command that's not finished yet.
        self.client.map(|client| client.command_complete(Ok(())));
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> Screen for Lpm013m126<'a, A, P, S>
where
    Self: 'static,
{
    fn get_resolution(&self) -> (usize, usize) {
        (176, 176)
    }

    fn get_pixel_format(&self) -> ScreenPixelFormat {
        ScreenPixelFormat::Mono
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
            return Err(ErrorCode::INVAL);
        }

        let frame = WriteFrame {
            row: y as u16,
            column: x as u16,
            width: width as u16,
            height: height as u16,
        };

        let mut new_state = None;
        let ret = match self.state.get() {
            State::Uninitialized | State::Off => Err(ErrorCode::OFF),
            State::InitializingPixelMemory | State::InitializingRest => Err(ErrorCode::BUSY),
            State::Idle(..) => {
                new_state = Some(State::Idle(frame));
                Ok(())
            }
            State::Writing(..) => {
                new_state = Some(State::Writing(frame));
                Ok(())
            }
            State::Bug => Err(ErrorCode::FAIL),
        };

        self.command_complete_callback.set();

        if let Some(new_state) = new_state {
            self.state.set(new_state);
        }

        ret
    }

    fn write(&self, buffer: &'static mut [u8], len: usize) -> Result<(), ErrorCode> {
        let ret = match self.state.get() {
            State::Uninitialized | State::Off => Err(ErrorCode::OFF),
            State::InitializingPixelMemory | State::InitializingRest => Err(ErrorCode::BUSY),
            State::Idle(frame) => {
                self.frame_buffer
                    .take()
                    .map_or(Err(ErrorCode::NOMEM), |mut frame_buffer| {
                        // TODO: reject if buffer is shorter than frame
                        frame_buffer.blit(&buffer[..cmp::min(buffer.len(), len)], &frame);
                        let send_buf = FrameBuffer::with_raw_rows(
                            frame_buffer,
                            frame.row,
                            frame.row + frame.height,
                        );

                        let sent = self.spi.read_write_bytes(send_buf, None, send_buf.len());
                        let (ret, new_state) = match sent {
                            Ok(()) => (Ok(()), State::Writing(frame)),
                            Err((e, buf, _)) => {
                                self.frame_buffer.replace(FrameBuffer::new(buf));
                                (Err(e), State::Idle(frame))
                            }
                        };
                        self.state.set(new_state);
                        ret
                    })
            }
            State::Writing(..) => Err(ErrorCode::BUSY),
            State::Bug => Err(ErrorCode::FAIL),
        };

        self.buffer.replace(buffer);

        match self.state.get() {
            State::Writing(..) => {}
            _ => self.call_write_complete(ret),
        };

        ret
    }

    fn write_continue(&self, buffer: &'static mut [u8], len: usize) -> Result<(), ErrorCode> {
        // Not TODO: this can be avoided entirely
        // at the cost of a minor layering violation.
        // https://github.com/tock/tock/pull/3011#issuecomment-1087766745
        self.write(buffer, len)?;
        // TODO: move position
        Ok(())
    }

    fn set_client(&self, client: Option<&'static dyn ScreenClient>) {
        if let Some(client) = client {
            self.client.set(client);
        } else {
            self.client.clear();
        }
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

    fn set_brightness(&self, _brightness: usize) -> Result<(), ErrorCode> {
        // TODO: add LED PWM
        Err(ErrorCode::NOSUPPORT)
    }

    fn set_invert(&self, _inverted: bool) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> AlarmClient for Lpm013m126<'a, A, P, S>
where
    Self: 'static,
{
    fn alarm(&self) {
        match self.state.get() {
            State::InitializingRest => {
                // Better flip it once too many than go out of spec
                // by stretching the flip period.
                self.extcomin.toggle();
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
                        State::Idle(
                            // The HIL doesn't specify the initial frame
                            WriteFrame {
                                row: 0,
                                column: 0,
                                width: 176,
                                height: 176,
                            },
                        )
                    },
                );

                self.state.set(new_state);

                if let State::Idle(..) = new_state {
                    self.client.map(|client| client.screen_is_ready());
                }
            }
            State::Idle(..) | State::Writing(..) => {
                self.extcomin.toggle();
                self.arm_alarm();
            }
            other => {
                debug!(
                    "LPM013M126 driver got alarm in unexpected state {:?}",
                    other
                );
                self.state.set(State::Bug);
            }
        };
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> SpiMasterClient for Lpm013m126<'a, A, P, S> {
    fn read_write_done(
        &self,
        write_buffer: SubmitBuffer<'static>,
        _read_buffer: Option<&'static mut [u8]>,
        _len: usize,
        status: Result<(), ErrorCode>,
    ) {
        self.frame_buffer.replace(FrameBuffer::new(write_buffer));
        self.state.set(match self.state.get() {
            State::InitializingPixelMemory => {
                // Rather than initialize them separately, wait longer and do both
                // for 2 reasons:
                // 1. the upper limit of waiting is only specified for both,
                // 2. and state flipping code is annoying and bug-friendly.
                self.disp.set();
                self.extcomin.set();
                let delay = self.alarm.ticks_from_us(200);
                self.alarm.set_alarm(self.alarm.now(), delay);
                State::InitializingRest
            }
            State::Writing(frame) => State::Idle(frame),
            // can't get more buggy than buggy
            other => {
                debug!(
                    "LPM013M126 received unexpected SPI complete in state {:?}",
                    other
                );
                State::Bug
            }
        });

        // Device frame buffer is now up to date, return pixel buffer to client.
        self.client.map(|client| {
            self.buffer
                .take()
                .map(|buf| client.write_complete(buf, status))
        });
    }
}

// DeferredCall requires a unique client for each DeferredCall so that different callbacks
// can be distinguished.
struct ReadyCallbackHandler<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> {
    lpm: OptionalCell<&'a Lpm013m126<'a, A, P, S>>,
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> ReadyCallbackHandler<'a, A, P, S> {
    fn new() -> Self {
        Self {
            lpm: OptionalCell::empty(),
        }
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> DeferredCallClient
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

struct CommandCompleteCallbackHandler<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> {
    lpm: OptionalCell<&'a Lpm013m126<'a, A, P, S>>,
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> CommandCompleteCallbackHandler<'a, A, P, S> {
    fn new() -> Self {
        Self {
            lpm: OptionalCell::empty(),
        }
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> DeferredCallClient
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

struct WriteCompleteCallbackHandler<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> {
    lpm: OptionalCell<&'a Lpm013m126<'a, A, P, S>>,
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> WriteCompleteCallbackHandler<'a, A, P, S> {
    fn new() -> Self {
        Self {
            lpm: OptionalCell::empty(),
        }
    }
}

impl<'a, A: Alarm<'a>, P: Pin, S: SpiMasterDevice> DeferredCallClient
    for WriteCompleteCallbackHandler<'a, A, P, S>
where
    Self: 'static,
{
    fn handle_deferred_call(&self) {
        self.lpm.map(|l| l.handle_write_complete_callback());
    }

    fn register(&'static self) {
        self.lpm.map(|l| l.write_complete_callback.register(self));
    }
}
