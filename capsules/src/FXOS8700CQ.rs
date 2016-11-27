use core::cell::Cell;
use kernel::{AppId, Callback, Driver};
use kernel::common::take_cell::TakeCell;
use kernel::hil::i2c::{I2CDevice, I2CClient, Error};

pub static mut BUFFER: [u8; 6] = [0; 6];

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
    XyzDataCfg = 0x0e,
    CtrlReg1 = 0x2a,
}

#[derive(Clone,Copy,PartialEq)]
enum State {
    Disabled,

    /// Enable sensor
    Active,

    /// Reading acceleration
    ReadingAcceleration,

    /// Enabling
    Enabling,

    /// Disabling
    Disabling(usize, usize, usize),
}

pub struct FXOS8700CQ<'a> {
    i2c: &'a I2CDevice,
    scale: Cell<u8>,
    state: Cell<State>,
    buffer: TakeCell<&'static mut [u8]>,
    callback: Cell<Option<Callback>>,
}

impl<'a> FXOS8700CQ<'a> {
    pub fn new(i2c: &'a I2CDevice, buffer: &'static mut [u8]) -> FXOS8700CQ<'a> {
        // setup and return struct
        FXOS8700CQ {
            i2c: i2c,
            scale: Cell::new(DEFAULT_SCALE),
            state: Cell::new(State::Disabled),
            buffer: TakeCell::new(buffer),
            callback: Cell::new(None),
        }
    }

    fn start_read_accel(&self, scale: u8) {
        // enable and configure FXOS8700CQ
        if self.state.get() == State::Disabled {
            self.buffer.take().map(|buf| {
                // turn on i2c
                self.i2c.enable();
                // configure accelerometer scale
                // TODO
                // buf[0] = Registers::XYZ_Data_CFG as u8;
                // buf[1] = scale as u8;
                // self.i2c.write(buf, 2);

                // TODO configure magnetometer

                // set to active mode
                buf[0] = Registers::CtrlReg1 as u8;
                // self.i2c.read(buf, 2);
                // buf[1] = buf[1] | 0x01;
                buf[1] = 0x01;
                self.i2c.write(buf, 2);
                self.state.set(State::Active);
            });
        }
    }
}

impl<'a> I2CClient for FXOS8700CQ<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: Error) {
        match self.state.get() { 
            State::Enabling => {
                buffer[0] = Registers::OutXLSB as u8;
                self.i2c.write_read(buffer, 1, 6); // write byte of register we want to read from
                self.state.set(State::ReadingAcceleration);
            }
            State::ReadingAcceleration => {
                // self.i2c.read(buffer, 6); // read 6 bytes for accel
                // let x = (((buffer[0] as usize) << 8) + buffer[1]) as usize;
                // let y = (((buffer[2] as usize) << 8) + buffer[3]) as usize;
                // let z = (((buffer[4] as usize) << 8) + buffer[5]) as usize;
                self.state.set(State::Disabling(buffer[0] as usize,
                                                buffer[2] as usize,
                                                buffer[4] as usize));
            }
            State::Disabling(x, y, z) => {
                self.i2c.disable();
                self.state.set(State::Disabled);
                self.buffer.replace(buffer);
                self.callback.get().map(|mut cb| cb.schedule(x, y, z));
            }
            _ => {}
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
                // self.start_read_accel(DEFAULT_SCALE);
                0
            }
            _ => -1,
        }
    }
}
