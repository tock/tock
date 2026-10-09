// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Driver for the Sensirion SHTC3 Temperature and Humidity Sensor.
//!
//! <https://sensirion.com/products/catalog/SHTC3/>
//!
//! This is a lower power version of the SHT3 sensor.

use core::cell::Cell;
use kernel::ErrorCode;
use kernel::hil::i2c;
use kernel::hil::time::{self, Alarm, ConvertTicks};
use kernel::utilities::cells::{OptionalCell, TakeCell};

/// I2C address.
pub const BASE_ADDR: u8 = 0x70;

/// SHTC3 command codes (Tables 9, 10, and 11 in the datasheet). Each
/// command is sent as two bytes, most-significant byte first.
mod command {
    /// Wake the sensor up from sleep mode. Required before any other
    /// command; the sensor is asleep on power-up.
    pub const WAKEUP: u16 = 0x3517;
    /// Send the sensor back to sleep, for lowest power consumption.
    pub const SLEEP: u16 = 0xB098;
    /// Measure both temperature and humidity, normal power mode.
    ///
    /// Reads temperature first, with clock stretching disabled (the
    /// sensor NACKs reads until the measurement is ready, rather than
    /// holding the clock line low while it measures).
    pub const MEASURE_NORMAL_T_FIRST_NO_STRETCH: u16 = 0x7866;
}

/// Time to wait after sending [`command::WAKEUP`] before the sensor is
/// guaranteed ready for the next command.
///
/// (Table 5, `t_PU`: typ. 180 us, max. 240 us), rounded up generously
/// since Tock alarms commonly can't resolve sub-millisecond delays.
const WAKE_UP_DELAY_MS: u32 = 1;

/// Time to wait after sending the measurement command before the result
/// is ready to read (Table 5, `t_MEAS`, normal mode: typ. 10.8 ms, max.
/// 12.1 ms), with margin.
const MEASURING_DELAY_MS: u32 = 15;

/// Number of bytes in one measurement result: a 2-byte temperature word
/// and its CRC, followed by a 2-byte humidity word and its CRC.
const MEASUREMENT_LEN: usize = 6;

/// CRC-8 with polynomial 0x31 (x^8 + x^5 + x^4 + 1), initialized to
/// 0xFF, as specified in Table 16 of the datasheet. Sensirion uses this
/// same checksum across its whole humidity/temperature sensor family.
fn crc8(data: &[u8]) -> u8 {
    let polynomial = 0x31;
    let mut crc = 0xff;

    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if (crc & 0x80) != 0 {
                crc = crc << 1 ^ polynomial;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    /// Wake-up command written; waiting for the I2C write to finish.
    Waking,
    /// Waiting out `WAKE_UP_DELAY_MS` before issuing the measurement
    /// command.
    WakeDelay,
    /// Measurement command written; waiting for the I2C write to finish.
    Measuring,
    /// Waiting out `MEASURING_DELAY_MS` before the result can be read.
    MeasuringDelay,
    /// Reading back the 6-byte measurement result.
    ReadingMeasurement,
    /// Put the sensor back to sleep.
    Sleeping,
}

pub struct Shtc3<'a, A: Alarm<'a>, I: i2c::I2CDevice<'a>> {
    i2c: &'a I,
    alarm: &'a A,
    humidity_client: OptionalCell<&'a dyn kernel::hil::sensors::HumidityClient>,
    temperature_client: OptionalCell<&'a dyn kernel::hil::sensors::TemperatureClient>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,

    /// Whether the client asked for a temperature reading.
    temperature_wanted: Cell<bool>,
    /// Whether the client asked for a humidity reading.
    humidity_wanted: Cell<bool>,
    /// Holds a temperature reading until it can be delivered to the client.
    pending_temperature: Cell<Option<Result<i32, ErrorCode>>>,
    /// Holds a humidity reading until it can be delivered to the client.
    pending_humidity: Cell<Option<usize>>,
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice<'a>> Shtc3<'a, A, I> {
    pub fn new(i2c: &'a I, buffer: &'static mut [u8], alarm: &'a A) -> Shtc3<'a, A, I> {
        Shtc3 {
            i2c,
            alarm,
            humidity_client: OptionalCell::empty(),
            temperature_client: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
            temperature_wanted: Cell::new(false),
            humidity_wanted: Cell::new(false),
            pending_temperature: Cell::new(None),
            pending_humidity: Cell::new(None),
        }
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        if self.temperature_wanted.get() {
            Err(ErrorCode::BUSY)
        } else if self.state.get() == State::Idle {
            let result = self.start_measurement();
            if result.is_ok() {
                self.temperature_wanted.set(true);
            }
            result
        } else {
            self.temperature_wanted.set(true);
            Ok(())
        }
    }

    fn read_humidity(&self) -> Result<(), ErrorCode> {
        if self.humidity_wanted.get() {
            Err(ErrorCode::BUSY)
        } else if self.state.get() == State::Idle {
            let result = self.start_measurement();
            if result.is_ok() {
                self.humidity_wanted.set(true);
            }
            result
        } else {
            self.humidity_wanted.set(true);
            Ok(())
        }
    }

    /// Write a two-byte command out to the sensor, taking `self.buffer`
    /// to do so.
    fn write_command(&self, command: u16) -> Result<(), ErrorCode> {
        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
            buffer[0] = (command >> 8) as u8;
            buffer[1] = (command & 0xff) as u8;

            match self.i2c.write(buffer, 2) {
                Ok(()) => Ok(()),
                Err((error, data)) => {
                    self.buffer.replace(data);
                    Err(error.into())
                }
            }
        })
    }

    fn read_measurement(&self) -> Result<(), ErrorCode> {
        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
            match self.i2c.read(buffer, MEASUREMENT_LEN) {
                Ok(()) => Ok(()),
                Err((error, data)) => {
                    self.buffer.replace(data);
                    Err(error.into())
                }
            }
        })
    }

    fn start_measurement(&self) -> Result<(), ErrorCode> {
        self.i2c.enable();
        self.state.set(State::Waking);

        let result = self.write_command(command::WAKEUP);
        if result.is_err() {
            self.state.set(State::Idle);
            self.i2c.disable();
        }
        result
    }

    /// Abort the in-progress cycle on an I2C error: disable the bus,
    /// stash the error for whichever client(s) had a reading pending, and
    /// return to `Idle` before delivering it (see `deliver_pending`).
    fn abort(&self, error: ErrorCode) {
        if self.temperature_wanted.get() {
            self.pending_temperature.set(Some(Err(error)));
        }
        if self.humidity_wanted.get() {
            self.pending_humidity.set(Some(usize::MAX));
        }

        self.finish();
    }

    /// Parse the 6-byte measurement result, checking each value's CRC,
    /// and stash it in `pending_temperature` and `pending_humidity`.
    fn store_measurement(&self) {
        self.buffer.map(|buffer| {
            let result = if crc8(&buffer[0..2]) == buffer[2] {
                let raw = ((buffer[0] as u32) << 8) | (buffer[1] as u32);
                // T[centi-degrees C] = -4500 + 17500 * raw / 65536.
                Ok(((4375 * raw) >> 14) as i32 - 4500)
            } else {
                Err(ErrorCode::FAIL)
            };
            self.pending_temperature.set(Some(result));

            let value = if crc8(&buffer[3..5]) == buffer[5] {
                let raw = ((buffer[3] as u32) << 8) | (buffer[4] as u32);
                // RH[centi-%] = 10000 * raw / 65536.
                ((625 * raw) >> 12) as usize
            } else {
                usize::MAX
            };
            self.pending_humidity.set(Some(value));
        });
    }

    /// Deliver whichever of `pending_temperature`/`pending_humidity` had
    /// a client waiting on it.
    fn deliver_pending(&self) {
        if self.temperature_wanted.get() {
            self.temperature_wanted.set(false);
            if let Some(result) = self.pending_temperature.take() {
                self.temperature_client
                    .map(|client| client.callback(result));
            }
        }
        if self.humidity_wanted.get() {
            self.humidity_wanted.set(false);
            if let Some(value) = self.pending_humidity.take() {
                self.humidity_client.map(|client| client.callback(value));
            }
        }
    }

    /// Disable the I2C bus, return to `Idle`, and deliver whatever
    /// reading(s) ended up pending.
    fn finish(&self) {
        self.i2c.disable();
        self.state.set(State::Idle);
        self.deliver_pending();
    }

    fn continue_state_machine(&self) {
        match self.state.get() {
            State::Idle => {}
            State::Waking => {
                self.state.set(State::WakeDelay);
                self.alarm
                    .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(WAKE_UP_DELAY_MS));
            }
            State::WakeDelay => {
                let res = self.write_command(command::MEASURE_NORMAL_T_FIRST_NO_STRETCH);
                if let Err(error) = res {
                    self.abort(error);
                } else {
                    self.state.set(State::Measuring);
                }
            }
            State::Measuring => {
                self.state.set(State::MeasuringDelay);
                self.alarm.set_alarm(
                    self.alarm.now(),
                    self.alarm.ticks_from_ms(MEASURING_DELAY_MS),
                );
            }
            State::MeasuringDelay => {
                let res = self.read_measurement();
                if let Err(error) = res {
                    self.abort(error);
                } else {
                    self.state.set(State::ReadingMeasurement);
                }
            }
            State::ReadingMeasurement => {
                self.store_measurement();

                let res = self.write_command(command::SLEEP);
                if let Err(_error) = res {
                    self.finish();
                } else {
                    self.state.set(State::Sleeping);
                }
            }
            State::Sleeping => {
                self.finish();
            }
        }
    }
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice<'a>> time::AlarmClient for Shtc3<'a, A, I> {
    fn alarm(&self) {
        self.continue_state_machine();
    }
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice<'a>> i2c::I2CClient for Shtc3<'a, A, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        self.buffer.replace(buffer);

        if let Err(error) = status {
            if self.state.get() == State::Sleeping {
                // The measurement itself already succeeded and is sitting in
                // `pending_temperature`/`pending_humidity`; only this best-effort
                // sleep command failed, which doesn't change what gets delivered.
                self.finish();
            } else {
                self.abort(error.into());
            }
        } else {
            self.continue_state_machine();
        }
    }
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice<'a>> kernel::hil::sensors::TemperatureDriver<'a>
    for Shtc3<'a, A, I>
{
    fn set_client(&self, client: &'a dyn kernel::hil::sensors::TemperatureClient) {
        self.temperature_client.set(client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        self.read_temperature()
    }
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice<'a>> kernel::hil::sensors::HumidityDriver<'a>
    for Shtc3<'a, A, I>
{
    fn set_client(&self, client: &'a dyn kernel::hil::sensors::HumidityClient) {
        self.humidity_client.set(client);
    }

    fn read_humidity(&self) -> Result<(), ErrorCode> {
        self.read_humidity()
    }
}
