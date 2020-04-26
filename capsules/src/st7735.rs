use crate::driver;
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::framebuffer::{self, ScreenFormat, ScreenRotation};
use kernel::hil::spi;
use kernel::hil::time::{self, Alarm, Frequency};
use kernel::ReturnCode;

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

static SW_RESET: Command = Command {
    id: 0x01,
    parameters: None,
    delay: 120,
};

static SLEEP_IN: Command = Command {
    id: 0x10,
    parameters: None,
    delay: 255,
};

static SLEEP_OUT: Command = Command {
    id: 0x11,
    parameters: None,
    delay: 255,
};

static PARTIAL_ON: Command = Command {
    id: 0x12,
    parameters: None,
    delay: 0,
};

static NORMAL_ON: Command = Command {
    id: 0x13,
    parameters: None,
    delay: 0,
};

static INVERSION_ON: Command = Command {
    id: 0x21,
    parameters: None,
    delay: 0,
};

static INVERSION_OFF: Command = Command {
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

static DISPLAY_ON: Command = Command {
    id: 0x29,
    /// Default Parameters: GamaSet
    parameters: None,
    delay: 255,
};

static DISPLAY_OFF: Command = Command {
    id: 0x28,
    parameters: None,
    delay: 255,
};

static CASET: Command = Command {
    id: 0x2A,
    /// Default Parameters: XS[15:8], XS[7:0], XE[15:8], XE[7,0] (128x160)
    parameters: Some(&[0, 0, 0, 127]),
    delay: 255,
};

static RASET: Command = Command {
    id: 0x2B,
    /// Default Parameters: YS[15:8], YS[7:0], YE[15:8], YE[7,0] (128x160)
    parameters: Some(&[0, 0, 0, 159]),
    delay: 255,
};

static RAM_WR: Command = Command {
    id: 0x2C,
    /// Default Parameters: data to write
    parameters: Some(&[]),
    delay: 255,
};

type CommandSequence = &'static [&'static Command];

static INIT_SEQUENCE: CommandSequence = &[&SW_RESET, &SLEEP_OUT];

#[derive(Copy, Clone, PartialEq)]
enum Status {
    Idle,
    SendCommand,
    Delay,
}

pub struct ST7735<'a, A: Alarm<'a>> {
    spi: &'a dyn spi::SpiMasterDevice,
    alarm: &'a A,
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
        buffer: &'static mut [u8],
    ) -> ST7735<'a, A> {
        spi.configure(
            spi::ClockPolarity::IdleLow,
            spi::ClockPhase::SampleTrailing,
            1_000_000,
        );
        ST7735 {
            alarm: alarm,
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
                    len = len + 1;
                    buffer[len] = *parameter;
                }
            }
        });
        self.send_command(cmd, len);
    }

    fn send_command(&self, cmd: &'static Command, len: usize) {
        self.command.set(cmd);
        self.status.set(Status::SendCommand);
        self.buffer.take().map(|buffer| {
            buffer[0] = cmd.id;
            self.spi.read_write_bytes(buffer, None, len);
        });
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
            Status::SendCommand => {
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
            }
            _ => {
                panic!("ST7735 status Idle");
            }
        };
    }

    pub fn init(&self) -> ReturnCode {
        self.send_sequence(INIT_SEQUENCE)
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
