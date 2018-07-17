//! Driver for the ST LPS25HB pressure sensor.
//!
//! <http://www.st.com/en/mems-and-sensors/lps25hb.html>
//!
//! Usage
//! -----
//!
//! ```rust
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
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::gpio;
use kernel::hil::i2c;
use kernel::{AppId, Callback, Driver, ReturnCode};

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x70004;

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

pub struct LPS25HB<'a> {
    i2c: &'a i2c::I2CDevice,
    interrupt_pin: &'a gpio::Pin,
    callback: OptionalCell<Callback>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
}

impl LPS25HB<'a> {
    pub fn new(
        i2c: &'a i2c::I2CDevice,
        interrupt_pin: &'a gpio::Pin,
        buffer: &'static mut [u8],
    ) -> LPS25HB<'a> {
        // setup and return struct
        LPS25HB {
            i2c: i2c,
            interrupt_pin: interrupt_pin,
            callback: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
        }
    }

    pub fn read_whoami(&self) {
        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            self.i2c.enable();

            buf[0] = Registers::WhoAmI as u8;
            self.i2c.write(buf, 1);
            self.state.set(State::SelectWhoAmI);
        });
    }

    pub fn take_measurement(&self) {
        self.interrupt_pin.make_input();
        self.interrupt_pin
            .enable_interrupt(0, gpio::InterruptMode::RisingEdge);

        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            self.i2c.enable();

            buf[0] = Registers::CtrlReg1 as u8 | REGISTER_AUTO_INCREMENT;
            buf[1] = 0;
            buf[2] = 0;
            buf[3] = 0;
            buf[4] = CTRL_REG4_INTERRUPT1_DATAREADY;
            self.i2c.write(buf, 5);
            self.state.set(State::TakeMeasurementInit);
        });
    }
}

impl i2c::I2CClient for LPS25HB<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        match self.state.get() {
            State::SelectWhoAmI => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadingWhoAmI);
            }
            State::ReadingWhoAmI => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::TakeMeasurementInit => {
                buffer[0] = Registers::PressOutXl as u8 | REGISTER_AUTO_INCREMENT;
                self.i2c.write(buffer, 1);
                self.state.set(State::TakeMeasurementClear);
            }
            State::TakeMeasurementClear => {
                self.i2c.read(buffer, 3);
                self.state.set(State::TakeMeasurementConfigure);
            }
            State::TakeMeasurementConfigure => {
                buffer[0] = Registers::CtrlReg1 as u8 | REGISTER_AUTO_INCREMENT;
                buffer[1] = CTRL_REG1_POWER_ON | CTRL_REG1_BLOCK_DATA_ENABLE;
                buffer[2] = CTRL_REG2_ONE_SHOT;
                self.i2c.write(buffer, 3);
                self.state.set(State::Done);
            }
            State::ReadMeasurement => {
                self.i2c.read(buffer, 3);
                self.state.set(State::GotMeasurement);
            }
            State::GotMeasurement => {
                let pressure = (((buffer[2] as u32) << 16)
                    | ((buffer[1] as u32) << 8)
                    | (buffer[0] as u32)) as u32;

                // Returned as microbars
                let pressure_ubar = (pressure * 1000) / 4096;

                self.callback
                    .map(|cb| cb.schedule(pressure_ubar as usize, 0, 0));

                buffer[0] = Registers::CtrlReg1 as u8;
                buffer[1] = 0;
                self.i2c.write(buffer, 2);
                self.interrupt_pin.disable_interrupt();
                self.state.set(State::Done);
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

impl gpio::Client for LPS25HB<'a> {
    fn fired(&self, _: usize) {
        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            self.i2c.enable();

            // select sensor voltage register and read it
            buf[0] = Registers::PressOutXl as u8 | REGISTER_AUTO_INCREMENT;
            self.i2c.write(buf, 1);
            self.state.set(State::ReadMeasurement);
        });
    }
}

impl Driver for LPS25HB<'a> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // Set a callback
            0 => {
                // Set callback function
                self.callback.insert(callback);
                ReturnCode::SUCCESS
            }
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, _: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            // Take a pressure measurement
            1 => {
                self.take_measurement();
                ReturnCode::SUCCESS
            }
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
