//! SyscallDriver for SHT3x Temperature and Humidity Sensor
//!
//! Author: Cosmin Daniel Radu <cosmindanielradu19@gmail.com>
//!
//!

use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::hil::i2c;
use kernel::hil::time::{self, Alarm, ConvertTicks};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

pub static BASE_ADDR: u8 = 0x44;

#[repr(u16)]
enum_from_primitive! {
    enum Registers {
        /// Measurement High Repeatability with Clock Stretch Enabled
        MEASHIGHREPSTRETCH = 0x2C06,
        /// Measurement Medium Repeatability with Clock Stretch Enabled
        MEASMEDREPSTRETCH = 0x2C0D,
        /// Measurement Low Repeatability with Clock Stretch Enabled
        MEASLOWREPSTRETCH = 0x2C10,
        /// Measurement High Repeatability with Clock Stretch Disabled
        MEASHIGHREP = 0x2400,
        /// Measurement Medium Repeatability with Clock Stretch Disabled
        MEASMEDREP = 0x240B,
        /// Measurement Low Repeatability with Clock Stretch Disabled
        MEASLOWREP = 0x2416,
        /// Read Out of Status Register
        READSTATUS = 0xF32D,
        /// Clear Status
        CLEARSTATUS = 0x3041,
        /// Soft Reset
        SOFTRESET = 0x30A2,
        /// Heater Enable
        HEATEREN = 0x306D,
        /// Heater Disable
        HEATERDIS = 0x3066,
        /// Status Register Heater Bit
        REGHEATERBIT = 0x0d,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    Read,
    ReadData,
}

fn crc8(data: &[u8]) -> u8 {
    let polynomial = 0x31;
    let mut crc = 0xff;

    for x in 0..data.len() {
        crc ^= data[x as usize] as u8;
        for _i in 0..8 {
            if (crc & 0x80) != 0 {
                crc = crc << 1 ^ polynomial;
            } else {
                crc = crc << 1;
            }
        }
    }
    crc
}

pub struct SHT3x<'a, A: Alarm<'a>> {
    i2c: &'a dyn i2c::I2CDevice,
    humidity_client: OptionalCell<&'a dyn kernel::hil::sensors::HumidityClient>,
    temperature_client: OptionalCell<&'a dyn kernel::hil::sensors::TemperatureClient>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    read_temp: Cell<bool>,
    read_hum: Cell<bool>,
    alarm: &'a A,
}

impl<'a, A: Alarm<'a>> SHT3x<'a, A> {
    pub fn new(
        i2c: &'a dyn i2c::I2CDevice,
        buffer: &'static mut [u8],
        alarm: &'a A,
    ) -> SHT3x<'a, A> {
        SHT3x {
            i2c: i2c,
            humidity_client: OptionalCell::empty(),
            temperature_client: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
            read_temp: Cell::new(false),
            read_hum: Cell::new(false),
            alarm: alarm,
        }
    }

    fn read_humidity(&self) -> Result<(), ErrorCode> {
        if self.read_hum.get() == true {
            Err(ErrorCode::BUSY)
        } else {
            if self.state.get() == State::Idle {
                self.read_hum.set(true);
                self.read_temp_hum()
            } else {
                self.read_hum.set(true);
                Ok(())
            }
        }
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        if self.read_temp.get() == true {
            Err(ErrorCode::BUSY)
        } else {
            if self.state.get() == State::Idle {
                self.read_temp.set(true);
                self.read_temp_hum()
            } else {
                self.read_temp.set(true);
                Ok(())
            }
        }
    }

    fn read_temp_hum(&self) -> Result<(), ErrorCode> {
        self.buffer.take().map_or_else(
            || panic!("SHT3x No buffer available!"),
            |buffer| {
                self.state.set(State::Read);
                self.i2c.enable();

                buffer[0] = ((Registers::MEASHIGHREP as u16) >> 8) as u8;
                buffer[1] = ((Registers::MEASHIGHREP as u16) & 0xff) as u8;

                // TODO verify errors
                let _ = self.i2c.write(buffer, 2);

                Ok(())
            },
        )
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for SHT3x<'a, A> {
    fn alarm(&self) {
        let state = self.state.get();
        match state {
            State::Read => {
                self.state.set(State::ReadData);
                self.buffer.take().map_or_else(
                    || panic!("SHT3x No buffer available!"),
                    |buffer| {
                        let _res = self.i2c.read(buffer, 6);
                    },
                );
            }
            _ => {
                // This should never happen
                panic!("SHT31 Invalid alarm!");
            }
        }
    }
}

impl<'a, A: Alarm<'a>> i2c::I2CClient for SHT3x<'a, A> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        match status {
            Ok(()) => {
                let state = self.state.get();

                match state {
                    State::ReadData => {
                        if self.read_temp.get() == true {
                            self.read_temp.set(false);
                            if crc8(&buffer[0..2]) == buffer[2] {
                                let mut stemp = buffer[0] as u32;
                                stemp = stemp << 8;
                                stemp = stemp | buffer[1] as u32;
                                stemp = ((4375 * stemp) >> 14) - 4500;
                                self.temperature_client
                                    .map(|cb| cb.callback(stemp as usize));
                            } else {
                                self.temperature_client.map(|cb| cb.callback(usize::MAX));
                            }
                        }
                        if self.read_hum.get() == true {
                            self.read_hum.set(false);
                            if crc8(&buffer[3..5]) == buffer[5] {
                                let mut shum = buffer[3] as u32;
                                shum = shum << 8;
                                shum = shum | buffer[4] as u32;
                                shum = (625 * shum) >> 12;
                                self.humidity_client.map(|cb| cb.callback(shum as usize));
                            } else {
                                self.humidity_client.map(|cb| cb.callback(usize::MAX));
                            }
                        }
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                    }
                    State::Read => {
                        self.buffer.replace(buffer);
                        let interval = self.alarm.ticks_from_ms(20);
                        self.alarm.set_alarm(self.alarm.now(), interval);
                    }
                    _ => {}
                }
            }
            _ => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                if self.read_temp.get() == true {
                    self.read_temp.set(false);
                    self.temperature_client.map(|cb| cb.callback(usize::MAX));
                }
                if self.read_hum.get() == true {
                    self.read_hum.set(false);
                    self.humidity_client.map(|cb| cb.callback(usize::MAX));
                }
            }
        }
    }
}

impl<'a, A: Alarm<'a>> kernel::hil::sensors::HumidityDriver<'a> for SHT3x<'a, A> {
    fn set_client(&self, client: &'a dyn kernel::hil::sensors::HumidityClient) {
        self.humidity_client.set(client);
    }

    fn read_humidity(&self) -> Result<(), ErrorCode> {
        self.read_humidity()
    }
}

impl<'a, A: Alarm<'a>> kernel::hil::sensors::TemperatureDriver<'a> for SHT3x<'a, A> {
    fn set_client(&self, client: &'a dyn kernel::hil::sensors::TemperatureClient) {
        self.temperature_client.set(client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        self.read_temperature()
    }
}
