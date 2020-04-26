use crate::driver;
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::framebuffer::{self, ScreenFormat, ScreenRotation};
use kernel::hil::spi;
use kernel::hil::gpio;
use kernel::hil::time::{self, Alarm, Frequency};
use kernel::ReturnCode;
use kernel::debug;

/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::St7735 as usize;

pub const BUFFER_SIZE: usize = 1024;
pub static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

struct Command {
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
    delay: 255,
};

static RASET: Command = Command {
    id: 0x2B,
    /// Default Parameters: YS[15:8], YS[7:0], YE[15:8], YE[7,0] (128x160)
    parameters: Some(&[0, 0, 0, 0x9F]),
    delay: 255,
};

static RAM_WR: Command = Command {
    id: 0x2C,
    /// Default Parameters: data to write
    parameters: Some(&[]),
    delay: 255,
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
    parameters: Some(&[0xC8]),
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
    parameters: Some(&[0x02, 0x1c, 0x07, 0x12,
        0x37, 0x32, 0x29, 0x2d,
        0x29, 0x25, 0x2B, 0x39,
        0x00, 0x01, 0x03, 0x10]),
    delay: 0,
};

static GMCTRN1: Command = Command {
    id: 0xE1,
    /// Default Parameters: 
    parameters: Some(&[0x03, 0x1d, 0x07, 0x06,
        0x2E, 0x2C, 0x29, 0x2D,
        0x2E, 0x2E, 0x37, 0x3F,
        0x00, 0x00, 0x02, 0x10]),
    delay: 0,
};

type CommandSequence = &'static [&'static Command];

static INIT_SEQUENCE: CommandSequence = &[&SWRESET, &SLPOUT, &FRMCTR1, &FRMCTR2, &FRMCTR3, &INVCTR, &PWCTR1, &PWCTR2, &PWCTR3, &PWCTR4, &PWCTR5, &VMCTR1, &INVOFF, &MADCTL, &COLMOD, &CASET, &RASET, &GMCTRP1, &GMCTRN1, &NORON, &DISPON];

#[derive(Copy, Clone, PartialEq)]
enum Status {
    Idle,
    Init,
    Reset1,
    Reset2,
    Reset3,
    Reset4,
    SendCommand (usize),
    SendParameters (usize),
    Delay,
}

pub struct ST7735<'a, A: Alarm<'a>> {
    spi: &'a dyn spi::SpiMasterDevice,
    alarm: &'a A,
    dc: &'a dyn gpio::Pin,
    reset: &'a dyn gpio::Pin,
    status: Cell<Status>,
    width: Cell<usize>,
    height: Cell<usize>,

    sequence_to_send: OptionalCell<CommandSequence>,
    position_in_sequence: Cell<usize>,
    command: Cell<&'static Command>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a, A: Alarm<'a>> ST7735<'a, A> {
    pub fn new(
        spi: &'a dyn spi::SpiMasterDevice,
        alarm: &'a A,
        dc: &'a dyn gpio::Pin,
        reset: &'a dyn gpio::Pin,
        buffer: &'static mut [u8],
    ) -> ST7735<'a, A> {
        spi.configure(
            spi::ClockPolarity::IdleLow,
            spi::ClockPhase::SampleTrailing,
            1_000_000,
        );
        ST7735 {
            alarm: alarm,

            dc: dc,
            reset: reset,
            spi: spi,

            status: Cell::new(Status::Idle),
            width: Cell::new(120),
            height: Cell::new(160),

            sequence_to_send: OptionalCell::empty(),
            position_in_sequence: Cell::new(0),
            command: Cell::new(&NOP),
            buffer: TakeCell::new(buffer),
        }
    }

    fn send_sequence(&self, sequence: CommandSequence) -> ReturnCode {
        if self.status.get() == Status::Idle {
            self.sequence_to_send.replace(sequence);
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
        self.buffer.map(|buffer| {
            buffer[0] = cmd.id;
            if let Some(parameters) = cmd.parameters {
                for parameter in parameters.iter() {
                    buffer[len+1] = *parameter;
                    len = len + 1;
                }
            }
        });
        self.send_command(cmd, len);
    }

    fn send_command(&self, cmd: &'static Command, len: usize) {
        debug! ("send command {} ({})", cmd.id, len);
        self.command.set(cmd);
        self.status.set(Status::SendCommand (len));
        self.dc.clear ();
        self.buffer.take().map(|buffer| {
            buffer[0] = cmd.id;
            self.spi.read_write_bytes(buffer, None, 1);
        });
    }

    fn send_parameters(&self, len: usize) {
        debug! ("send parameters ({})", len);
        self.status.set (Status::SendParameters(len));
        if len > 0 {
            self.buffer.take().map(|buffer| {
                // shift parameters
                for i in 0 .. len {
                    buffer[i] = buffer[i+1];
                }
                self.dc.set ();
                self.spi.read_write_bytes(buffer, None, len);
            });
        }
        else
        {
            self.do_next_op ();
        }
    }

    fn do_next_op(&self) {
        match self.status.get() {
            Status::Delay => {
                self.sequence_to_send.map_or_else(
                    || self.status.set(Status::Idle),
                    |sequence| {
                        // sendf next command in the sequence
                        let position = self.position_in_sequence.get();
                        self.position_in_sequence
                            .set(self.position_in_sequence.get() + 1);
                        if position < sequence.len() {
                            self.send_command_with_default_parameters(sequence[position]);
                        } else {
                            self.sequence_to_send.clear();
                            self.status.set(Status::Idle);
                        }
                    },
                );
            }
            Status::SendCommand(parameters_length) => {
                self.send_parameters (parameters_length);
            }
            Status::SendParameters(_) => {
                self.dc.clear ();
                let mut delay = self.command.get().delay as u32;
                debug! ("delay {}", delay);
                if delay > 0 {
                    if delay == 255 {
                        delay = 500;
                    }
                    self.set_delay(delay, Status::Delay)
                } else {
                    self.status.set(Status::Delay);
                    self.do_next_op();
                }
            }
            Status::Reset1 => {
                self.spi.hold_low ();
                self.send_command_with_default_parameters (&NOP);
                self.set_delay (10, Status::Reset2);
            }
            Status::Reset2 => {
                self.reset.set ();
                self.set_delay (500, Status::Reset3);
            }
            Status::Reset3 => {
                self.reset.clear ();
                self.set_delay (500, Status::Reset4);
            }
            Status::Reset4 => {
                self.reset.set ();
                self.set_delay (500, Status::Init);
            }
            Status::Init => {
                self.spi.release_low ();
                self.status.set (Status::Idle);
                self.send_sequence (INIT_SEQUENCE);
            }
            _ => {
                panic!("ST7735 status Idle");
            }
        };
    }

    pub fn init(&self) {
        self.status.set (Status::Reset1);
        self.do_next_op ();
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

impl<'a, A: Alarm<'a>> framebuffer::Configuration for ST7735<'a, A> {
    fn set_size(&self, _width: usize, _height: usize) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn set_format(&self, _format: ScreenFormat) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn set_rotation(&self, _format: ScreenRotation) -> ReturnCode {
        ReturnCode::ENOSUPPORT
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
