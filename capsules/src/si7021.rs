//! Silicon Labs SI7021 Temperature/Humidity Sensor
//!
//! https://www.silabs.com/products/sensors/humidity-sensors/Pages/si7013-20-21.aspx

use core::cell::Cell;
use kernel::{AppId, Callback, Driver, ReturnCode};

use kernel::common::take_cell::TakeCell;
use kernel::hil::i2c;
use kernel::hil::time;
use kernel::hil::time::Frequency;

// Buffer to use for I2C messages
pub static mut BUFFER: [u8; 14] = [0; 14];

#[allow(dead_code)]
enum Registers {
    MeasRelativeHumidityHoldMode = 0xe5,
    MeasRelativeHumidityNoHoldMode = 0xf5,
    MeasTemperatureHoldMode = 0xe3,
    MeasTemperatureNoHoldMode = 0xf3,
    ReadTemperaturePreviousRHMeasurement = 0xe0,
    Reset = 0xfe,
    WriteRHTUserRegister1 = 0xe6,
    ReadRHTUserRegister1 = 0xe7,
    WriteHeaterControlRegister = 0x51,
    ReadHeaterControlRegister = 0x11,
    ReadElectronicIdByteOneA = 0xfa,
    ReadElectronicIdByteOneB = 0x0f,
    ReadElectronicIdByteTwoA = 0xfc,
    ReadElectronicIdByteTwoB = 0xc9,
    ReadFirmwareVersionA = 0x84,
    ReadFirmwareVersionB = 0xb8,
}

/// States of the I2C protocol with the LPS331AP.
#[derive(Clone,Copy,PartialEq)]
enum State {
    Idle,

    /// States to read the internal ID
    SelectElectronicId1,
    ReadElectronicId1,
    SelectElectronicId2,
    ReadElectronicId2,

    /// States to take the current measurement
    TakeMeasurementInit,
    ReadRhMeasurement,
    ReadTempMeasurement,
    GotMeasurement,
}

pub struct SI7021<'a, A: time::Alarm + 'a> {
    i2c: &'a i2c::I2CDevice,
    alarm: &'a A,
    callback: Cell<Option<Callback>>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a, A: time::Alarm + 'a> SI7021<'a, A> {
    pub fn new(i2c: &'a i2c::I2CDevice, alarm: &'a A, buffer: &'static mut [u8]) -> SI7021<'a, A> {
        // setup and return struct
        SI7021 {
            i2c: i2c,
            alarm: alarm,
            callback: Cell::new(None),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
        }
    }

    pub fn read_id(&self) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            buffer[0] = Registers::ReadElectronicIdByteOneA as u8;
            buffer[1] = Registers::ReadElectronicIdByteOneB as u8;
            self.i2c.write(buffer, 2);
            self.state.set(State::SelectElectronicId1);
        });
    }

    pub fn take_measurement(&self) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            buffer[0] = Registers::MeasRelativeHumidityNoHoldMode as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::TakeMeasurementInit);
        });
    }
}

impl<'a, A: time::Alarm + 'a> i2c::I2CClient for SI7021<'a, A> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        match self.state.get() {
            State::SelectElectronicId1 => {
                self.i2c.read(buffer, 8);
                self.state.set(State::ReadElectronicId1);
            }
            State::ReadElectronicId1 => {
                buffer[6] = buffer[0];
                buffer[7] = buffer[1];
                buffer[8] = buffer[2];
                buffer[9] = buffer[3];
                buffer[10] = buffer[4];
                buffer[11] = buffer[5];
                buffer[12] = buffer[6];
                buffer[13] = buffer[7];
                buffer[0] = Registers::ReadElectronicIdByteTwoA as u8;
                buffer[1] = Registers::ReadElectronicIdByteTwoB as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::SelectElectronicId2);
            }
            State::SelectElectronicId2 => {
                self.i2c.read(buffer, 6);
                self.state.set(State::ReadElectronicId2);
            }
            State::ReadElectronicId2 => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::TakeMeasurementInit => {

                let interval = (20 as u32) * <A::Frequency>::frequency() / 1000;

                let tics = self.alarm.now().wrapping_add(interval);
                self.alarm.set_alarm(tics);

                // Now wait for timer to expire
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::ReadRhMeasurement => {
                buffer[2] = buffer[0];
                buffer[3] = buffer[1];
                buffer[0] = Registers::ReadTemperaturePreviousRHMeasurement as u8;
                self.i2c.write(buffer, 1);
                self.state.set(State::ReadTempMeasurement);
            }
            State::ReadTempMeasurement => {
                self.i2c.read(buffer, 2);
                self.state.set(State::GotMeasurement);
            }
            State::GotMeasurement => {

                // Temperature in hundredths of degrees centigrade
                let temp_raw = (((buffer[0] as u32) << 8) | (buffer[1] as u32)) as u32;
                let temp = (((temp_raw * 17572) / 65536) - 4685) as i16;

                // Humidity in hundredths of percent
                let humidity_raw = (((buffer[2] as u32) << 8) | (buffer[3] as u32)) as u32;
                let humidity = (((humidity_raw * 125 * 100) / 65536) - 600) as u16;

                self.callback.get().map(|mut cb| cb.schedule(temp as usize, humidity as usize, 0));

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            _ => {}
        }
    }
}

impl<'a, A: time::Alarm + 'a> time::Client for SI7021<'a, A> {
    fn fired(&self) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            self.i2c.read(buffer, 2);
            self.state.set(State::ReadRhMeasurement);
        });
    }
}

impl<'a, A: time::Alarm + 'a> Driver for SI7021<'a, A> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // Set a callback
            0 => {
                // Set callback function
                self.callback.set(Some(callback));
                ReturnCode::SUCCESS
            }
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, _: usize, _: AppId) -> ReturnCode {
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
