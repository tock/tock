//! Driver for the MLX90614 Infrared Thermometer.
//!
//! SMBus Interface
//!
//! Usage
//! -----
//!
//! ```rust
//! let mux_i2c = components::i2c::I2CMuxComponent::new(&ibex::i2c::I2C)
//!     .finalize(components::i2c_mux_component_helper!());
//!
//! let mlx90614 = components::mlx90614::Mlx90614I2CComponent::new()
//!    .finalize(components::mlx90614_i2c_component_helper!(mux_i2c));
//! ```
//!

use crate::driver;
use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::register_bitfields;
use kernel::hil::i2c::{self, Error};
use kernel::hil::sensors;
use kernel::{AppId, Callback, Driver, ReturnCode};

/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Mlx90614 as usize;

register_bitfields![u16,
    CONFIG [
        IIR OFFSET(0) NUMBITS(3) [],
        DUAL OFFSET(6) NUMBITS(1) [],
        FIR OFFSET(8) NUMBITS(3) [],
        GAIN OFFSET(11) NUMBITS(3) []
    ]
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    Idle,
    IsPresent,
    ReadAmbientTemp,
    ReadObjTemp,
}

enum_from_primitive! {
    enum Mlx90614Registers {
        RAW1 = 0x04,
        RAW2 = 0x05,
        TA = 0x06,
        TOBJ1 = 0x07,
        TOBJ2 = 0x08,
        EMISSIVITY = 0x24,
        CONFIG = 0x25,
    }
}

pub struct Mlx90614SMBus<'a> {
    smbus_temp: &'a dyn i2c::SMBusDevice,
    callback: OptionalCell<Callback>,
    temperature_client: OptionalCell<&'a dyn sensors::TemperatureClient>,
    buffer: TakeCell<'static, [u8]>,
    state: Cell<State>,
}

impl<'a> Mlx90614SMBus<'_> {
    pub fn new(
        smbus_temp: &'a dyn i2c::SMBusDevice,
        buffer: &'static mut [u8],
    ) -> Mlx90614SMBus<'a> {
        Mlx90614SMBus {
            smbus_temp,
            callback: OptionalCell::empty(),
            temperature_client: OptionalCell::empty(),
            buffer: TakeCell::new(buffer),
            state: Cell::new(State::Idle),
        }
    }

    fn is_present(&self) {
        self.state.set(State::IsPresent);
        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            buf[0] = Mlx90614Registers::RAW1 as u8;
            self.smbus_temp.smbus_write_read(buf, 1, 1).unwrap();
        });
    }

    fn read_ambient_temperature(&self) {
        self.state.set(State::ReadAmbientTemp);
        self.buffer.take().map(|buf| {
            buf[0] = Mlx90614Registers::TA as u8;
            self.smbus_temp.smbus_write_read(buf, 1, 1).unwrap();
        });
    }

    fn read_object_temperature(&self) {
        self.state.set(State::ReadObjTemp);
        self.buffer.take().map(|buf| {
            buf[0] = Mlx90614Registers::TOBJ1 as u8;
            self.smbus_temp.smbus_write_read(buf, 1, 2).unwrap();
        });
    }
}

impl<'a> i2c::I2CClient for Mlx90614SMBus<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], error: Error) {
        match self.state.get() {
            State::Idle => {
                self.buffer.replace(buffer);
            }
            State::IsPresent => {
                let present = if error == Error::CommandComplete && buffer[0] == 60 {
                    true
                } else {
                    false
                };

                self.callback.map(|callback| {
                    callback.schedule(if present { 1 } else { 0 }, 0, 0);
                });
                self.buffer.replace(buffer);
                self.state.set(State::Idle);
            }
            State::ReadAmbientTemp | State::ReadObjTemp => {
                let mut temp: usize = 0;

                let values = if error == Error::CommandComplete {
                    // Convert to centi celsius
                    temp = ((buffer[0] as usize | (buffer[1] as usize) << 8) * 2) - 27300;
                    self.temperature_client.map(|client| {
                        client.callback(temp as usize);
                    });
                    true
                } else {
                    self.temperature_client.map(|client| {
                        client.callback(0);
                    });
                    false
                };
                if values {
                    self.callback.map(|callback| {
                        callback.schedule(temp, 0, 0);
                    });
                } else {
                    self.callback.map(|callback| {
                        callback.schedule(0, 0, 0);
                    });
                }
                self.buffer.replace(buffer);
                self.state.set(State::Idle);
            }
        }
    }
}

impl<'a> Driver for Mlx90614SMBus<'a> {
    fn command(&self, command_num: usize, _data1: usize, _data2: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,
            // Check is sensor is correctly connected
            1 => {
                if self.state.get() == State::Idle {
                    self.is_present();
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::EBUSY
                }
            }
            // Read Ambient Temperature
            2 => {
                if self.state.get() == State::Idle {
                    self.read_ambient_temperature();
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::EBUSY
                }
            }
            // Read Object Temperature
            3 => {
                if self.state.get() == State::Idle {
                    self.read_object_temperature();
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::EBUSY
                }
            }
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
            0 /* set the one shot callback */ => {
				self.callback.insert(callback);
				ReturnCode::SUCCESS
			},
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a> sensors::TemperatureDriver<'a> for Mlx90614SMBus<'a> {
    fn set_client(&self, temperature_client: &'a dyn sensors::TemperatureClient) {
        self.temperature_client.replace(temperature_client);
    }

    fn read_temperature(&self) -> ReturnCode {
        self.read_object_temperature();
        ReturnCode::SUCCESS
    }
}
