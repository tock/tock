//! ST7735 SPI Screen
//!
//! Usage
//! -----
//!
//! ```rust
//!
//! let tft = components::st7735::ST7735Component::new(alarm_mux).finalize(
//!     components::st7735_component_helper!(
//!         // spi type
//!         stm32f4xx::spi::Spi,
//!         // chip select
//!         stm32f4xx::gpio::PinId::PE03,
//!         // spi mux
//!         spi_mux,
//!         // timer type
//!         stm32f4xx::tim2::Tim2,
//!         // dc pin
//!         stm32f4xx::gpio::PinId::PA00.get_pin().as_ref().unwrap(),
//!         // reset pin
//!         stm32f4xx::gpio::PinId::PA00.get_pin().as_ref().unwrap()
//!     )
//! );
//! ```

use crate::driver;
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::framebuffer::{self, ScreenClient, ScreenPixelFormat, ScreenRotation};
use kernel::hil::gpio;
use kernel::hil::spi;
use kernel::hil::time::{self, Alarm, Frequency};
use kernel::ReturnCode;
use kernel::{AppId, Callback, Driver};

/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::St7735 as usize;

const BUFFER_SIZE: usize = 40980;
pub static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

#[derive(PartialEq)]
pub struct Command {
    id: u8,
    parameters: Option<&'static [u8]>,
    delay: u8,
}

static NOP: Command = Command {
    id: 0x00,
    parameters: None,
    delay: 0,
};

static SWRESET: Command = Command {
    id: 0x01,
    parameters: None,
    delay: 150,
};

#[allow(dead_code)]
static SLPIN: Command = Command {
    id: 0x10,
    parameters: None,
    delay: 255,
};

static SLPOUT: Command = Command {
    id: 0x11,
    parameters: None,
    delay: 255,
};

#[allow(dead_code)]
static PLTON: Command = Command {
    id: 0x12,
    parameters: None,
    delay: 0,
};

static NORON: Command = Command {
    id: 0x13,
    parameters: None,
    delay: 10,
};

static INVON: Command = Command {
    id: 0x21,
    parameters: None,
    delay: 0,
};

static INVOFF: Command = Command {
    id: 0x20,
    parameters: None,
    delay: 0,
};

#[allow(dead_code)]
static GAMSET: Command = Command {
    id: 0x26,
    /// Default parameters: Gama Set
    parameters: Some(&[0]),
    delay: 0,
};

static DISPON: Command = Command {
    id: 0x29,
    /// Default Parameters: GamaSet
    parameters: None,
    delay: 100,
};

static DISPOFF: Command = Command {
    id: 0x28,
    parameters: None,
    delay: 100,
};

static CASET: Command = Command {
    id: 0x2A,
    /// Default Parameters: XS[15:8], XS[7:0], XE[15:8], XE[7,0] (128x160)
    parameters: Some(&[0, 0, 0, 0x7F]),
    delay: 0,
};

static RASET: Command = Command {
    id: 0x2B,
    /// Default Parameters: YS[15:8], YS[7:0], YE[15:8], YE[7,0] (128x160)
    parameters: Some(&[0, 0, 0, 0x9F]),
    delay: 0,
};

static RAMWR: Command = Command {
    id: 0x2C,
    /// Default Parameters: data to write
    parameters: Some(&[]),
    delay: 0,
};

static FRMCTR1: Command = Command {
    id: 0xB1,
    /// Default Parameters:
    parameters: Some(&[0x01, 0x2C, 0x2D]),
    delay: 0,
};

static FRMCTR2: Command = Command {
    id: 0xB2,
    /// Default Parameters:
    parameters: Some(&[0x01, 0x2C, 0x2D]),
    delay: 0,
};

static FRMCTR3: Command = Command {
    id: 0xB3,
    /// Default Parameters:
    parameters: Some(&[0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D]),
    delay: 0,
};

static INVCTR: Command = Command {
    id: 0xB4,
    /// Default Parameters:
    parameters: Some(&[0x07]),
    delay: 0,
};

static PWCTR1: Command = Command {
    id: 0xC0,
    /// Default Parameters:
    parameters: Some(&[0xA2, 0x02, 0x84]),
    delay: 0,
};

static PWCTR2: Command = Command {
    id: 0xC1,
    /// Default Parameters:
    parameters: Some(&[0xC5]),
    delay: 0,
};

static PWCTR3: Command = Command {
    id: 0xC2,
    /// Default Parameters:
    parameters: Some(&[0x0A, 0x00]),
    delay: 0,
};

static PWCTR4: Command = Command {
    id: 0xC3,
    /// Default Parameters:
    parameters: Some(&[0x8A, 0x2A]),
    delay: 0,
};

static PWCTR5: Command = Command {
    id: 0xC4,
    /// Default Parameters:
    parameters: Some(&[0x8A, 0xEE]),
    delay: 0,
};

static VMCTR1: Command = Command {
    id: 0xC5,
    /// Default Parameters:
    parameters: Some(&[0x0E]),
    delay: 0,
};

static MADCTL: Command = Command {
    id: 0x36,
    /// Default Parameters:
    parameters: Some(&[0x00]),
    delay: 0,
};

static COLMOD: Command = Command {
    id: 0x3A,
    /// Default Parameters:
    parameters: Some(&[0x05]),
    delay: 0,
};

static GMCTRP1: Command = Command {
    id: 0xE0,
    /// Default Parameters:
    parameters: Some(&[
        0x02, 0x1c, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2d, 0x29, 0x25, 0x2B, 0x39, 0x00, 0x01, 0x03,
        0x10,
    ]),
    delay: 0,
};

static GMCTRN1: Command = Command {
    id: 0xE1,
    /// Default Parameters:
    parameters: Some(&[
        0x03, 0x1d, 0x07, 0x06, 0x2E, 0x2C, 0x29, 0x2D, 0x2E, 0x2E, 0x37, 0x3F, 0x00, 0x00, 0x02,
        0x10,
    ]),
    delay: 0,
};

pub type CommandSequence = &'static [SendCommand];

macro_rules! default_parameters_sequence {
    ($($cmd:expr),+) => {
        [$(SendCommand::Default($cmd), )+]
    }
}

static INIT_SEQUENCE: [SendCommand; 20] = default_parameters_sequence!(
    &SWRESET, &SLPOUT, &FRMCTR1, &FRMCTR2, &FRMCTR3, &INVCTR, &PWCTR1, &PWCTR2, &PWCTR3, &PWCTR4,
    &PWCTR5, &VMCTR1, &INVOFF, &MADCTL, &COLMOD, &CASET, &RASET, &GMCTRP1, &GMCTRN1, &NORON
);

static WRITE_PIXEL: [SendCommand; 3] = [
    SendCommand::Position(&CASET, 1, 4),
    SendCommand::Position(&RASET, 5, 4),
    SendCommand::Position(&RAMWR, 9, 2),
];

const SEQUENCE_BUFFER_SIZE: usize = 24;
pub static mut SEQUENCE_BUFFER: [SendCommand; SEQUENCE_BUFFER_SIZE] =
    [SendCommand::Nop; SEQUENCE_BUFFER_SIZE];

#[derive(Copy, Clone, PartialEq)]
enum Status {
    Idle,
    Init,
    Reset1,
    Reset2,
    Reset3,
    Reset4,
    SendCommand(usize, usize, usize),
    SendCommandSlice(usize),
    Delay,
}
#[derive(Copy, Clone, PartialEq)]
pub enum SendCommand {
    Nop,
    Default(&'static Command),
    // first usize is the position in the buffer
    // second usize is the length in the buffer starting from the position
    Position(&'static Command, usize, usize),
    // first usize is the position in the buffer (4 bytes - repeat times, length bytes data)
    // second usize is the length in the buffer
    // third usize is the number of repeats
    Repeat(&'static Command, usize, usize, usize),
    // usize is length
    Slice(&'static Command),
}

pub struct ST7735<'a, A: Alarm<'a>> {
    spi: &'a dyn spi::SpiMasterDevice,
    alarm: &'a A,
    dc: &'a dyn gpio::Pin,
    reset: &'a dyn gpio::Pin,
    status: Cell<Status>,
    callback: OptionalCell<Callback>,
    width: Cell<usize>,
    height: Cell<usize>,

    client: OptionalCell<&'static dyn framebuffer::ScreenClient>,

    sequence_buffer: TakeCell<'static, [SendCommand]>,
    position_in_sequence: Cell<usize>,
    sequence_len: Cell<usize>,
    command: Cell<&'static Command>,
    buffer: TakeCell<'static, [u8]>,

    power_on: Cell<bool>,
}

impl<'a, A: Alarm<'a>> ST7735<'a, A> {
    pub fn new(
        spi: &'a dyn spi::SpiMasterDevice,
        alarm: &'a A,
        dc: &'a dyn gpio::Pin,
        reset: &'a dyn gpio::Pin,
        buffer: &'static mut [u8],
        sequence_buffer: &'static mut [SendCommand],
    ) -> ST7735<'a, A> {
        spi.configure(
            spi::ClockPolarity::IdleLow,
            spi::ClockPhase::SampleTrailing,
            4_000_000,
        );
        ST7735 {
            alarm: alarm,

            dc: dc,
            reset: reset,
            spi: spi,

            callback: OptionalCell::empty(),

            status: Cell::new(Status::Idle),
            width: Cell::new(128),
            height: Cell::new(160),

            client: OptionalCell::empty(),

            sequence_buffer: TakeCell::new(sequence_buffer),
            sequence_len: Cell::new(0),
            position_in_sequence: Cell::new(0),
            command: Cell::new(&NOP),
            buffer: TakeCell::new(buffer),

            power_on: Cell::new(false),
        }
    }

    fn send_sequence(&self, sequence: CommandSequence) -> ReturnCode {
        if self.status.get() == Status::Idle {
            let error = self.sequence_buffer.map_or_else(
                || panic!("st7735: send sequence has no sequence buffer"),
                |sequence_buffer| {
                    if sequence.len() <= sequence_buffer.len() {
                        self.sequence_len.set(sequence.len());
                        for (i, cmd) in sequence.iter().enumerate() {
                            sequence_buffer[i] = *cmd;
                        }
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::ENOMEM
                    }
                },
            );
            if error == ReturnCode::SUCCESS {
                self.send_sequence_buffer()
            } else {
                error
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn send_sequence_buffer(&self) -> ReturnCode {
        if self.status.get() == Status::Idle {
            self.position_in_sequence.set(0);
            // set status to delay so that do_next_op will send the next item in the sequence
            self.status.set(Status::Delay);
            self.do_next_op();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }

    fn send_command_with_default_parameters(&self, cmd: &'static Command) {
        let mut len = 0;
        self.buffer.map_or_else(
            || panic!("st7735: send parameters has no buffer"),
            |buffer| {
                buffer[0] = cmd.id;
                if let Some(parameters) = cmd.parameters {
                    for parameter in parameters.iter() {
                        buffer[len + 1] = *parameter;
                        len = len + 1;
                    }
                }
            },
        );
        self.send_command(cmd, 1, len, 1);
    }

    fn send_command(&self, cmd: &'static Command, position: usize, len: usize, repeat: usize) {
        self.command.set(cmd);
        self.status.set(Status::SendCommand(position, len, repeat));
        self.dc.clear();
        self.buffer.take().map_or_else(
            || panic!("st7735: send command has no buffer"),
            |buffer| {
                buffer[0] = cmd.id;
                self.spi.read_write_bytes(buffer, None, 1);
            },
        );
    }

    fn send_command_slice(&self, cmd: &'static Command) {
        self.command.set(cmd);
        self.dc.clear();
        self.status.set(Status::SendCommandSlice(1));
        self.buffer.take().map_or_else(
            || panic!("st7735: send command has no buffer"),
            |buffer| {
                buffer[0] = cmd.id;
                self.spi.read_write_bytes(buffer, None, 1);
            },
        );
    }

    fn send_parameters(&self, position: usize, len: usize, repeat: usize) {
        self.status.set(Status::SendCommand(0, len, repeat - 1));
        if len > 0 {
            self.buffer.take().map_or_else(
                || panic!("st7735: send parameters has no buffer"),
                |buffer| {
                    // shift parameters
                    if position > 0 {
                        for i in position..len + position {
                            buffer[i - position] = buffer[i];
                        }
                    }
                    self.dc.set();
                    self.spi.read_write_bytes(buffer, None, len);
                },
            );
        } else {
            self.do_next_op();
        }
    }

    fn send_parameters_slice(&self) {
        self.status.set(Status::SendCommandSlice(0));
        self.client.map_or_else(
            || panic!("st7735: no screen client"),
            |client| {
                self.buffer.take().map_or_else(
                    || panic!("st7735: send parameters has no buffer"),
                    |buffer| {
                        let len = client.fill_next_buffer_for_write(buffer);
                        if len > 0 {
                            self.status.set(Status::SendCommandSlice(len));
                            self.dc.set();
                            self.spi.read_write_bytes(buffer, None, len);
                        } else {
                            self.buffer.replace(buffer);
                            self.do_next_op();
                        }
                    },
                );
            },
        );
    }

    fn fill(&self, color: usize) -> ReturnCode {
        if self.status.get() == Status::Idle {
            // TODO check if buffer is available
            self.sequence_buffer.map_or_else(
                || panic!("st7735: fill has no sequence buffer"),
                |sequence| {
                    sequence[0] = SendCommand::Default(&CASET);
                    sequence[1] = SendCommand::Default(&RASET);
                    self.buffer.map_or_else(
                        || panic!("st7735: fill has no buffer"),
                        |buffer| {
                            let bytes = 128 * 160 * 2;
                            let buffer_space = (buffer.len() - 9) / 2 * 2;
                            let repeat = (bytes / buffer_space) + 1;
                            sequence[2] = SendCommand::Repeat(&RAMWR, 9, buffer_space, repeat);
                            for index in 0..(buffer_space / 2) {
                                buffer[9 + 2 * index] = ((color >> 8) & 0xFF) as u8;
                                buffer[9 + (2 * index + 1)] = color as u8;
                            }
                        },
                    );
                    self.sequence_len.set(3);
                },
            );
            self.send_sequence_buffer();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }

    fn rotation(&self, rotation: ScreenRotation) -> ReturnCode {
        if self.status.get() == Status::Idle {
            let rotation_bits = match rotation {
                ScreenRotation::Normal => 0x00,
                ScreenRotation::Rotated90 => 0x60,
                ScreenRotation::Rotated180 => 0xC0,
                ScreenRotation::Rotated270 => 0xA0,
            };
            match rotation {
                ScreenRotation::Normal | ScreenRotation::Rotated180 => {
                    self.width.set(128);
                    self.height.set(160);
                }
                ScreenRotation::Rotated90 | ScreenRotation::Rotated270 => {
                    self.width.set(160);
                    self.height.set(128);
                }
            };
            self.buffer.map_or_else(
                || panic!("st7735: set rotation has no buffer"),
                |buffer| {
                    buffer[1] =
                        rotation_bits | MADCTL.parameters.map_or(0, |parameters| parameters[0])
                },
            );
            self.send_command(&MADCTL, 1, 1, 1);
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }

    fn display_on(&self) -> ReturnCode {
        if self.status.get() == Status::Idle {
            if !self.power_on.get() {
                ReturnCode::EOFF
            } else {
                self.send_command_with_default_parameters(&DISPON);
                ReturnCode::SUCCESS
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn display_off(&self) -> ReturnCode {
        if self.status.get() == Status::Idle {
            if !self.power_on.get() {
                ReturnCode::EOFF
            } else {
                self.send_command_with_default_parameters(&DISPOFF);
                ReturnCode::SUCCESS
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn display_invert_on(&self) -> ReturnCode {
        if self.status.get() == Status::Idle {
            if !self.power_on.get() {
                ReturnCode::EOFF
            } else {
                self.send_command_with_default_parameters(&INVON);
                ReturnCode::SUCCESS
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn display_invert_off(&self) -> ReturnCode {
        if self.status.get() == Status::Idle {
            if !self.power_on.get() {
                ReturnCode::EOFF
            } else {
                self.send_command_with_default_parameters(&INVOFF);
                ReturnCode::SUCCESS
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn do_next_op(&self) {
        match self.status.get() {
            Status::Delay => {
                self.sequence_buffer.map_or_else(
                    || panic!("st7735: do next op has no sequence buffer"),
                    |sequence| {
                        // sendf next command in the sequence
                        let position = self.position_in_sequence.get();
                        self.position_in_sequence
                            .set(self.position_in_sequence.get() + 1);
                        if position < self.sequence_len.get() {
                            match sequence[position] {
                                SendCommand::Nop => {
                                    self.do_next_op();
                                }
                                SendCommand::Default(ref cmd) => {
                                    self.send_command_with_default_parameters(cmd);
                                }
                                SendCommand::Position(ref cmd, position, len) => {
                                    self.send_command(cmd, position, len, 1);
                                }
                                SendCommand::Repeat(ref cmd, position, len, repeat) => {
                                    self.send_command(cmd, position, len, repeat);
                                }
                                SendCommand::Slice(ref cmd) => {
                                    self.send_command_slice(cmd);
                                }
                            };
                        } else {
                            self.status.set(Status::Idle);
                            self.callback.map(|callback| {
                                callback.schedule(0, 0, 0);
                            });
                            if !self.power_on.get() {
                                self.client.map(|client| {
                                    self.power_on.set(true);
                                    client.screen_is_ready();
                                });
                            } else {
                                self.client.map(|client| {
                                    client.command_complete(ReturnCode::SUCCESS);
                                });
                            }
                        }
                    },
                );
            }
            Status::SendCommand(parameters_position, parameters_length, repeat) => {
                if repeat == 0 {
                    self.dc.clear();
                    let mut delay = self.command.get().delay as u32;
                    if delay > 0 {
                        if delay == 255 {
                            delay = 500;
                        }
                        self.set_delay(delay, Status::Delay)
                    } else {
                        self.status.set(Status::Delay);
                        self.do_next_op();
                    }
                } else {
                    self.send_parameters(parameters_position, parameters_length, repeat);
                }
            }
            Status::SendCommandSlice(len) => {
                if len == 0 {
                    self.dc.clear();
                    let mut delay = self.command.get().delay as u32;
                    if delay > 0 {
                        if delay == 255 {
                            delay = 500;
                        }
                        self.set_delay(delay, Status::Delay)
                    } else {
                        self.status.set(Status::Delay);
                        self.do_next_op();
                    }
                } else {
                    self.send_parameters_slice();
                }
            }
            Status::Reset1 => {
                self.send_command_with_default_parameters(&NOP);
                self.set_delay(10, Status::Reset2);
            }
            Status::Reset2 => {
                self.reset.set();
                self.set_delay(120, Status::Reset3);
            }
            Status::Reset3 => {
                self.reset.clear();
                self.set_delay(120, Status::Reset4);
            }
            Status::Reset4 => {
                self.reset.set();
                self.set_delay(120, Status::Init);
            }
            Status::Init => {
                self.status.set(Status::Idle);
                self.send_sequence(&INIT_SEQUENCE);
            }
            _ => {
                panic!("ST7735 status Idle");
            }
        };
    }

    fn set_memory_frame(
        &self,
        position: usize,
        sx: usize,
        sy: usize,
        ex: usize,
        ey: usize,
    ) -> ReturnCode {
        if sx <= self.width.get()
            && sy <= self.height.get()
            && ex <= self.width.get()
            && ey <= self.height.get()
            && sx <= ex
            && sy <= ey
        {
            if self.status.get() == Status::Idle {
                self.buffer.map_or_else(
                    || panic!("st7735: set memory frame has no buffer"),
                    |buffer| {
                        // CASET
                        buffer[position] = 0;
                        buffer[position + 1] = sx as u8;
                        buffer[position + 2] = 0;
                        buffer[position + 3] = ex as u8;
                        // RASET
                        buffer[position + 4] = 0;
                        buffer[position + 5] = sy as u8;
                        buffer[position + 6] = 0;
                        buffer[position + 7] = ey as u8;
                    },
                );
                ReturnCode::SUCCESS
            } else {
                ReturnCode::EBUSY
            }
        } else {
            ReturnCode::EINVAL
        }
    }

    // fn write_data(&self, data: &'static [u8], len: usize) -> ReturnCode {
    //     if self.status.get() == Status::Idle {
    //         self.buffer.map(|buffer| {
    //             // TODO verify length
    //             for position in 0..len {
    //                 buffer[position + 1] = data[position];
    //             }
    //         });
    //         self.send_command(&RAMWR, 1, len, 1);
    //         ReturnCode::SUCCESS
    //     } else {
    //         ReturnCode::EBUSY
    //     }
    // }

    fn write_pixel(&self, x: usize, y: usize, color: usize) -> ReturnCode {
        if x < self.width.get() && y < self.height.get() {
            if self.status.get() == Status::Idle {
                self.buffer.map_or_else(
                    || panic!("st7735: write pixel has no buffer"),
                    |buffer| {
                        // CASET
                        buffer[1] = 0;
                        buffer[2] = x as u8;
                        buffer[3] = 0;
                        buffer[4] = (x + 1) as u8;
                        // RASET
                        buffer[5] = 0;
                        buffer[6] = y as u8;
                        buffer[7] = 0;
                        buffer[8] = (y + 1) as u8;
                        // RAMWR
                        buffer[9] = ((color >> 8) & 0xFF) as u8;
                        buffer[10] = (color & 0xFF) as u8
                    },
                );
                self.send_sequence(&WRITE_PIXEL)
            } else {
                ReturnCode::EBUSY
            }
        } else {
            ReturnCode::EINVAL
        }
    }

    pub fn init(&self) -> ReturnCode {
        if self.status.get() == Status::Idle {
            self.status.set(Status::Reset1);
            self.do_next_op();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }

    /// set_delay sets an alarm and saved the next state after that.
    ///
    /// As argument, there are:
    ///  - the duration of the alarm in ms
    ///  - the status of the program after the alarm fires
    ///
    /// Example:
    ///  self.set_delay(10, Status::Idle);
    fn set_delay(&self, timer: u32, next_status: Status) {
        self.status.set(next_status);
        self.alarm.set_alarm(
            self.alarm
                .now()
                .wrapping_add(<A::Frequency>::frequency() / 1000 * timer),
        )
    }
}

impl<'a, A: Alarm<'a>> Driver for ST7735<'a, A> {
    fn command(&self, command_num: usize, data1: usize, data2: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,
            // reset
            1 => self.init(),
            // fill with color (data1)
            2 => self.fill(data1),
            // write pixel (x:data1[15:8], y:data1[7:0], color:data2)
            3 => self.write_pixel((data1 >> 8) & 0xFF, data1 & 0xFF, data2),
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => {
                self.callback.insert(callback);
                ReturnCode::SUCCESS
            }
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a, A: Alarm<'a>> framebuffer::ScreenSetup for ST7735<'a, A> {
    fn set_resolution(&self, resolution: (usize, usize)) -> ReturnCode {
        if resolution.0 == self.width.get() && resolution.1 == self.height.get() {
            ReturnCode::SUCCESS
        } else {
            ReturnCode::ENOSUPPORT
        }
    }

    fn set_pixel_format(&self, depth: ScreenPixelFormat) -> ReturnCode {
        if depth == ScreenPixelFormat::RGB_565 {
            // if not outstanding
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EINVAL
        }
    }

    fn set_rotation(&self, rotation: ScreenRotation) -> ReturnCode {
        self.rotation(rotation)
    }

    fn get_supported_resolutions(&self) -> usize {
        1
    }
    fn get_supported_resolution(&self, index: usize) -> Option<(usize, usize)> {
        match index {
            0 => Some((self.width.get(), self.height.get())),
            _ => None,
        }
    }

    fn get_supported_pixel_formats(&self) -> usize {
        1
    }
    fn get_supported_pixel_format(&self, index: usize) -> Option<ScreenPixelFormat> {
        match index {
            0 => Some(ScreenPixelFormat::RGB_565),
            _ => None,
        }
    }
}

impl<'a, A: Alarm<'a>> framebuffer::Screen for ST7735<'a, A> {
    fn get_resolution(&self) -> (usize, usize) {
        (self.width.get(), self.height.get())
    }

    fn get_pixel_format(&self) -> ScreenPixelFormat {
        ScreenPixelFormat::RGB_565
    }

    fn get_rotation(&self) -> ScreenRotation {
        ScreenRotation::Normal
    }

    fn write(&self, x: usize, y: usize, width: usize, height: usize) -> ReturnCode {
        if self.status.get() == Status::Idle {
            let buffer_len = self.buffer.map_or_else(
                || panic!("st7735: buffer is not available"),
                |buffer| buffer.len() - 1,
            );
            if buffer_len >= 9 {
                // set buffer
                let err = self.set_memory_frame(1, x, y, x + width, y + height);
                if err == ReturnCode::SUCCESS {
                    self.sequence_buffer.map_or_else(
                        || panic!("st7735: write no sequence buffer"),
                        |sequence| {
                            sequence[0] = SendCommand::Position(&CASET, 1, 4);
                            sequence[1] = SendCommand::Position(&RASET, 5, 4);
                            sequence[2] = SendCommand::Slice(&RAMWR);
                            self.sequence_len.set(3);
                        },
                    );
                    self.send_sequence_buffer();
                }
                err
            } else {
                ReturnCode::ENOMEM
            }
        } else {
            ReturnCode::EBUSY
        }
    }

    fn set_client(&self, client: Option<&'static dyn ScreenClient>) {
        if let Some(client) = client {
            self.client.set(client);
        } else {
            self.client.clear();
        }
    }

    fn on(&self) -> ReturnCode {
        self.display_on()
    }

    fn off(&self) -> ReturnCode {
        self.display_off()
    }

    fn invert_on(&self) -> ReturnCode {
        self.display_invert_on()
    }

    fn invert_off(&self) -> ReturnCode {
        self.display_invert_off()
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for ST7735<'a, A> {
    fn fired(&self) {
        self.do_next_op();
    }
}

impl<'a, A: Alarm<'a>> spi::SpiMasterClient for ST7735<'a, A> {
    fn read_write_done(
        &self,
        write_buffer: &'static mut [u8],
        _read_buffer: Option<&'static mut [u8]>,
        _len: usize,
    ) {
        self.buffer.replace(write_buffer);
        self.do_next_op();
    }
}
