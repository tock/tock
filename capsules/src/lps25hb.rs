//! SyscallDriver for the ST LPS25HB pressure sensor.
//!
//! <http://www.st.com/en/mems-and-sensors/lps25hb.html>
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let lps25hb_i2c = static_init!(I2CDevice, I2CDevice::new(i2c_bus, 0x5C));
//! let lps25hb = static_init!(
//!     capsules::lps25hb::LPS25HB<'static>,
//!     capsules::lps25hb::LPS25HB::new(lps25hb_i2c,
//!         &sam4l::gpio::PA[10],
//!         &mut capsules::lps25hb::BUFFER));
//! lps25hb_i2c.set_client(lps25hb);
//! sam4l::gpio::PA[10].set_client(lps25hb);
//! ```

use core::cell::Cell;

use kernel::errorcode::into_statuscode;
use kernel::grant::Grant;
use kernel::hil::gpio;
use kernel::hil::i2c;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Lps25hb as usize;

// Buffer to use for I2C messages
pub static mut BUFFER: [u8; 5] = [0; 5];

/// Register values
const REGISTER_AUTO_INCREMENT: u8 = 0x80;

const CTRL_REG1_POWER_ON: u8 = 0x80;
const CTRL_REG1_BLOCK_DATA_ENABLE: u8 = 0x04;
const CTRL_REG2_ONE_SHOT: u8 = 0x01;
const CTRL_REG4_INTERRUPT1_DATAREADY: u8 = 0x01;

#[allow(dead_code)]
enum Registers {
    RefPXl = 0x08,
    RefPL = 0x09,
    RefPH = 0x0a,
    WhoAmI = 0x0f,
    ResConf = 0x10,
    CtrlReg1 = 0x20,
    CtrlReg2 = 0x21,
    CtrlReg3 = 0x22,
    CtrlReg4 = 0x23,
    IntCfgReg = 0x24,
    IntSourceReg = 0x25,
    StatusReg = 0x27,
    PressOutXl = 0x28,
    PressOutL = 0x29,
    PressOutH = 0x2a,
    TempOutL = 0x2b,
    TempOutH = 0x2c,
    FifoCtrl = 0x2e,
    FifoStatus = 0x2f,
    ThsPL = 0x30,
    ThsPH = 0x31,
    RpdsL = 0x39,
    RpdsH = 0x3a,
}

/// States of the I2C protocol with the LPS25HB.
#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,

    /// Read the WHO_AM_I register. This should return 0xBB.
    SelectWhoAmI,
    ReadingWhoAmI,

    /// Process of taking a pressure measurement.
    /// Start with chip powered off
    TakeMeasurementInit,
    /// Then clear the current reading (just in case it exists)
    /// to reset the interrupt line.
    TakeMeasurementClear,
    /// Enable a single shot measurement with interrupt when data is ready.
    TakeMeasurementConfigure,

    /// Read the 3 pressure registers.
    ReadMeasurement,
    /// Calculate pressure and call the callback with the value.
    GotMeasurement,

    /// Disable I2C and release buffer
    Done,
}

#[derive(Default)]
pub struct App {}

pub struct LPS25HB<'a> {
    i2c: &'a dyn i2c::I2CDevice,
    interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    apps: Grant<App, 1>,
    owning_process: OptionalCell<ProcessId>,
}

impl<'a> LPS25HB<'a> {
    pub fn new(
        i2c: &'a dyn i2c::I2CDevice,
        interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
        buffer: &'static mut [u8],
        apps: Grant<App, 1>,
    ) -> Self {
        // setup and return struct
        Self {
            i2c: i2c,
            interrupt_pin: interrupt_pin,
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
            apps,
            owning_process: OptionalCell::empty(),
        }
    }

    pub fn read_whoami(&self) -> Result<(), ErrorCode> {
        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            // turn on i2c to send commands
            self.i2c.enable();

            buf[0] = Registers::WhoAmI as u8;

            if let Err((_error, buf)) = self.i2c.write(buf, 1) {
                self.buffer.replace(buf);
                self.i2c.disable();
                Err(_error.into())
            } else {
                self.state.set(State::SelectWhoAmI);
                Ok(())
            }
        })
    }

    pub fn take_measurement(&self) -> Result<(), ErrorCode> {
        self.interrupt_pin.make_input();
        self.interrupt_pin
            .enable_interrupts(gpio::InterruptEdge::RisingEdge);

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            // turn on i2c to send commands
            self.i2c.enable();

            buf[0] = Registers::CtrlReg1 as u8 | REGISTER_AUTO_INCREMENT;
            buf[1] = 0;
            buf[2] = 0;
            buf[3] = 0;
            buf[4] = CTRL_REG4_INTERRUPT1_DATAREADY;

            if let Err((_error, buf)) = self.i2c.write(buf, 5) {
                self.buffer.replace(buf);
                self.i2c.disable();
                Err(_error.into())
            } else {
                self.state.set(State::TakeMeasurementInit);
                Ok(())
            }
        })
    }
}

impl i2c::I2CClient for LPS25HB<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if status != Ok(()) {
            self.state.set(State::Idle);
            self.buffer.replace(buffer);
            self.owning_process.map(|pid| {
                let _ = self.apps.enter(*pid, |_app, upcalls| {
                    upcalls.schedule_upcall(0, 0, 0, 0).ok();
                });
            });
            return;
        }
        match self.state.get() {
            State::SelectWhoAmI => {
                if let Err((_error, buffer)) = self.i2c.read(buffer, 1) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buffer);
                } else {
                    self.state.set(State::ReadingWhoAmI);
                }
            }
            State::ReadingWhoAmI => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::TakeMeasurementInit => {
                buffer[0] = Registers::PressOutXl as u8 | REGISTER_AUTO_INCREMENT;
                if let Err((error, buffer)) = self.i2c.write(buffer, 1) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buffer);
                    self.owning_process.map(|pid| {
                        let _ = self.apps.enter(*pid, |_app, upcalls| {
                            upcalls
                                .schedule_upcall(0, into_statuscode(Err(error.into())), 0, 0)
                                .ok();
                        });
                    });
                } else {
                    self.state.set(State::TakeMeasurementClear);
                }
            }
            State::TakeMeasurementClear => {
                if let Err((error, buffer)) = self.i2c.read(buffer, 3) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buffer);
                    self.owning_process.map(|pid| {
                        let _ = self.apps.enter(*pid, |_app, upcalls| {
                            upcalls
                                .schedule_upcall(into_statuscode(Err(error.into())), 0, 0, 0)
                                .ok();
                        });
                    });
                } else {
                    self.state.set(State::TakeMeasurementConfigure);
                }
            }
            State::TakeMeasurementConfigure => {
                buffer[0] = Registers::CtrlReg1 as u8 | REGISTER_AUTO_INCREMENT;
                buffer[1] = CTRL_REG1_POWER_ON | CTRL_REG1_BLOCK_DATA_ENABLE;
                buffer[2] = CTRL_REG2_ONE_SHOT;

                if let Err((error, buffer)) = self.i2c.write(buffer, 3) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buffer);
                    self.owning_process.map(|pid| {
                        let _ = self.apps.enter(*pid, |_app, upcalls| {
                            upcalls
                                .schedule_upcall(into_statuscode(Err(error.into())), 0, 0, 0)
                                .ok();
                        });
                    });
                } else {
                    self.state.set(State::Done);
                }
            }
            State::ReadMeasurement => {
                if let Err((error, buffer)) = self.i2c.read(buffer, 3) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buffer);
                    self.owning_process.map(|pid| {
                        let _ = self.apps.enter(*pid, |_app, upcalls| {
                            upcalls
                                .schedule_upcall(into_statuscode(Err(error.into())), 0, 0, 0)
                                .ok();
                        });
                    });
                } else {
                    self.state.set(State::GotMeasurement);
                }
            }
            State::GotMeasurement => {
                let pressure = (((buffer[2] as u32) << 16)
                    | ((buffer[1] as u32) << 8)
                    | (buffer[0] as u32)) as u32;

                // Returned as microbars
                let pressure_ubar = (pressure * 1000) / 4096;

                self.owning_process.map(|pid| {
                    let _ = self.apps.enter(*pid, |_app, upcalls| {
                        upcalls
                            .schedule_upcall(0, pressure_ubar as usize, 0, 0)
                            .ok();
                    });
                });

                buffer[0] = Registers::CtrlReg1 as u8;
                buffer[1] = 0;

                if let Err((error, buffer)) = self.i2c.write(buffer, 2) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buffer);
                    self.owning_process.map(|pid| {
                        let _ = self.apps.enter(*pid, |_app, upcalls| {
                            upcalls
                                .schedule_upcall(into_statuscode(Err(error.into())), 0, 0, 0)
                                .ok();
                        });
                    });
                } else {
                    self.interrupt_pin.disable_interrupts();
                    self.state.set(State::Done);
                }
            }
            State::Done => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            _ => {}
        }
    }
}

impl gpio::Client for LPS25HB<'_> {
    fn fired(&self) {
        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            self.i2c.enable();

            // select sensor voltage register and read it
            buf[0] = Registers::PressOutXl as u8 | REGISTER_AUTO_INCREMENT;

            if let Err((_error, buf)) = self.i2c.write(buf, 1) {
                self.buffer.replace(buf);
                self.i2c.disable();
            } else {
                self.state.set(State::ReadMeasurement);
            }
        });
    }
}

impl SyscallDriver for LPS25HB<'_> {
    fn command(
        &self,
        command_num: usize,
        _: usize,
        _: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        if command_num == 0 {
            // Handle this first as it should be returned
            // unconditionally
            return CommandReturn::success();
        }
        // Check if this non-virtualized driver is already in use by
        // some (alive) process
        let match_or_empty_or_nonexistant = self.owning_process.map_or(true, |current_process| {
            self.apps
                .enter(*current_process, |_, _| current_process == &process_id)
                .unwrap_or(true)
        });
        if match_or_empty_or_nonexistant {
            self.owning_process.set(process_id);
        } else {
            return CommandReturn::failure(ErrorCode::NOMEM);
        }
        match command_num {
            // Take a pressure measurement
            1 => match self.take_measurement() {
                Ok(()) => CommandReturn::success(),
                Err(error) => CommandReturn::failure(error),
            },
            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
