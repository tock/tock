//! Driver for the ISL29035 digital light sensor

use main::{AppId, Callback, Driver};
use hil::i2c::{I2CDevice, I2CClient, Error};
use core::cell::Cell;
use common::take_cell::TakeCell;

pub static mut BUF : [u8; 3] = [0; 3];

#[derive(Copy,Clone,PartialEq)]
enum State {
    Disabled,
    Enabling,
    ReadingLI,
    Disabling(usize)
}

pub struct Isl29035<'a> {
    i2c: &'a I2CDevice,
    state: Cell<State>,
    buffer: TakeCell<&'static mut [u8]>,
    callback: Cell<Option<Callback>>,
}

impl<'a> Isl29035<'a> {
    pub fn new(i2c: &'a I2CDevice, buffer: &'static mut [u8])
            -> Isl29035<'a> {
        Isl29035 {
            i2c: i2c,
            state: Cell::new(State::Disabled),
            buffer: TakeCell::new(buffer),
            callback: Cell::new(None)
        }
    }

    pub fn start_read_lux(&self) {
        if self.state.get() == State::Disabled {
            self.buffer.take().map(|buf| {
                self.i2c.enable();
                buf[0] = 0;
                // CMD 1 Register:
                // Interrupt persist for 1 integration cycle (bits 0 & 1)
                // Measure ALS continuously (buts 5,6 & 7)
                // Bit 2 is the interrupt bit
                // Bits 3 & 4 are reserved
                buf[1] = 0b10100000;

                // CMD 2 Register:
                // Range 4000 (bits 0, 1)
                // ADC resolution 8-bit (bits 2,3)
                // Other bits are reserved
                buf[2] = 0b00001001;
                self.i2c.write(buf, 3);
                self.state.set(State::Enabling);
            });
        }
    }
}

impl<'a> Driver for Isl29035<'a> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 => {
                self.callback.set(Some(callback));
                0
            }
            _ => -1
        }
    }

    fn command(&self, command_num: usize, _arg1: usize, _: AppId) -> isize {
        match command_num {
            0 => {
                self.start_read_lux();
                0
            },
            _ => -1
        }
    }
}

impl<'a> I2CClient for Isl29035<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: Error) {
        // TODO(alevy): handle I2C errors
        match self.state.get() {
            State::Enabling => {
                buffer[0] = 0x02 as u8;
                self.i2c.write_read(buffer, 1, 2);
                self.state.set(State::ReadingLI);
            },
            State::ReadingLI => {
                // During configuration we set the ADC resolution to 8 bits and
                // the range to 4000.
                //
                // Since it's only 8 bits, we ignore the second byte of output.
                //
                // For a given Range and n (-bits of ADC resolution):
                // Lux = Data * (Range / 2^n)
                let data = buffer[0] as usize; //((buffer[1] as usize) << 8) | buffer[0] as usize;
                let lux = (data * 4000) >> 8;

                buffer[0] = 0;
                self.i2c.write(buffer, 2);
                self.state.set(State::Disabling(lux));
            },
            State::Disabling(lux) => {
                self.i2c.disable();
                self.state.set(State::Disabled);
                self.buffer.replace(buffer);
                self.callback.get().map(|mut cb| cb.schedule(lux, 0, 0));
            },
            _ => {}
        }
    }
}

