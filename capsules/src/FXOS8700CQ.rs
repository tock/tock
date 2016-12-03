use core::cell::Cell;
use kernel::{AppId, Callback, Driver};
use kernel::common::take_cell::TakeCell;
use kernel::hil::i2c::{I2CDevice, I2CClient, Error};

pub static mut BUF: [u8; 6] = [0; 6];

const DEFAULT_SCALE: u8 = 0x0;

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
    /// Sensor is disabled (but on)
    Disabled,

    /// Reading that sensor is present
    Enabling,

    /// Activate sensor for readings
    Activating,

    /// Deactivate sensor from readings
    Deactivating(usize, usize, usize),

    /// Reading accelerometer data
    ReadingAcceleration,
}

pub struct FXOS8700CQ<'a> {
    i2c: &'a I2CDevice,
    state: Cell<State>,
    buffer: TakeCell<&'static mut [u8]>,
    callback: Cell<Option<Callback>>,
}

impl<'a> FXOS8700CQ<'a> {
    pub fn new(i2c: &'a I2CDevice, buffer: &'static mut [u8]) -> FXOS8700CQ<'a> {
        FXOS8700CQ {
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

impl<'a> I2CClient for FXOS8700CQ<'a> {
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
                let x = (((buffer[0] as u16) << 8) | buffer[1] as u16) as usize;
                let y = (((buffer[2] as u16) << 8) | buffer[3] as u16) as usize;
                let z = (((buffer[4] as u16) << 8) | buffer[5] as u16) as usize;

                let x = ((x >> 2) * 976) / 1000;
                let y = ((y >> 2) * 976) / 1000;
                let z = ((z >> 2) * 976) / 1000;

                buffer[0] = 0; 
                self.i2c.write(buffer, 2); 
                self.state.set(State::Deactivating(x, y, z));
            }
            State::Deactivating(x, y, z) => {
                self.i2c.disable(); 
                self.state.set(State::Disabled);
                self.buffer.replace(buffer);
                self.callback.get().map(|mut cb| cb.schedule(x, y, z));
            }
        }
    }
}

impl<'a> Driver for FXOS8700CQ<'a> {
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
            0 => {
                // read acceleration
                self.start_read_accel();
                0
            }
            _ => -1,
        }
    }
}
