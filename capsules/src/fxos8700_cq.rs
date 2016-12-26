//! Driver for the FXOS8700CQ accelerometer
//! http://www.nxp.com/assets/documents/data/en/data-sheets/FXOS8700CQ.pdf
//! The driver provides x, y, and z acceleration data to a callback function.
//! To use readings from the sensor in userland, see FXOS8700CQ.h in libtock.

use core::cell::Cell;
use kernel::{AppId, Callback, Driver};
use kernel::common::take_cell::TakeCell;
use kernel::hil::i2c::{I2CDevice, I2CClient, Error};

pub static mut BUF: [u8; 6] = [0; 6];

#[allow(dead_code)]
enum Registers {
    SensorStatus = 0x00,
    OutXMSB = 0x01,
    OutXLSB = 0x02,
    OutYMSB = 0x03,
    OutYLSB = 0x04,
    OutZMSB = 0x05,
    OutZLSB = 0x06,
    XyzDataCfg = 0x0E,
    WhoAmI = 0x0D,
    CtrlReg1 = 0x2A,
}

#[derive(Clone,Copy,PartialEq)]
enum State {
    /// Sensor does not take acceleration readings
    Disabled,

    /// Verifying that sensor is present
    Enabling,

    /// Activate sensor to take readings
    Activating,

    /// Reading accelerometer data
    ReadingAcceleration,

    /// Deactivate sensor
    Deactivating(i16, i16, i16),
}

pub struct Fxos8700cq<'a> {
    i2c: &'a I2CDevice,
    state: Cell<State>,
    buffer: TakeCell<&'static mut [u8]>,
    callback: Cell<Option<Callback>>,
}

impl<'a> Fxos8700cq<'a> {
    pub fn new(i2c: &'a I2CDevice, buffer: &'static mut [u8]) -> Fxos8700cq<'a> {
        Fxos8700cq {
            i2c: i2c,
            state: Cell::new(State::Enabling),
            buffer: TakeCell::new(buffer),
            callback: Cell::new(None),
        }
    }

    fn start_read_accel(&self) {
        self.buffer.take().map(|buf| {
            self.i2c.enable();
            buf[0] = Registers::WhoAmI as u8;
            self.i2c.write_read(buf, 1, 1);
            self.state.set(State::Enabling);
        });
    }
}

impl<'a> I2CClient for Fxos8700cq<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: Error) {
        match self.state.get() {
            State::Disabled => {
                // self.i2c.disable();
            }
            State::Enabling => {
                buffer[0] = Registers::CtrlReg1 as u8; // CTRL_REG1
                buffer[1] = 1; // active
                self.i2c.write(buffer, 2);
                self.state.set(State::Activating);
            }
            State::Activating => {
                buffer[0] = Registers::OutXMSB as u8;
                self.i2c.write_read(buffer, 1, 6); // read 6 accel registers for xyz
                self.state.set(State::ReadingAcceleration);
            }
            State::ReadingAcceleration => {
                let x = (((buffer[0] as i16) << 8) | buffer[1] as i16) >> 2;
                let y = (((buffer[2] as i16) << 8) | buffer[3] as i16) >> 2;
                let z = (((buffer[4] as i16) << 8) | buffer[5] as i16) >> 2;

                let x = ((x as isize) * 244) / 1000;
                let y = ((y as isize) * 244) / 1000;
                let z = ((z as isize) * 244) / 1000;

                buffer[0] = 0;
                self.i2c.write(buffer, 2);
                self.state.set(State::Deactivating(x as i16, y as i16, z as i16));
            }
            State::Deactivating(x, y, z) => {
                self.i2c.disable();
                self.state.set(State::Disabled);
                self.buffer.replace(buffer);
                self.callback.get().map(|mut cb| cb.schedule(x as usize, y as usize, z as usize));
            }
        }
    }
}

impl<'a> Driver for Fxos8700cq<'a> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 => {
                self.callback.set(Some(callback));
                0
            }
            _ => -1,
        }
    }

    fn command(&self, command_num: usize, _arg1: usize, _: AppId) -> isize {
        match command_num {
            0 /* check if present */ => 0,
            1 => {
                // read acceleration
                self.start_read_accel();
                0
            }
            _ => -1,
        }
    }
}
