//! hil driver for Bmp280 Temperature and Pressure Sensor
//!
//! Written by Dorota <gihu.dcz@porcupinefactory.org>
//!
//! Based off the SHT3x code.
//!
//! Not implemented: pressure

use core::cell::Cell;
use kernel::debug;
use kernel::hil;
use kernel::hil::i2c;
use kernel::hil::time::{Alarm, ConvertTicks};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

pub static BASE_ADDR: u8 = 0x76;

/// Currently sized enough for temperature readings only.
pub const BUFFER_SIZE: usize = 6;

#[allow(non_camel_case_types)]
#[allow(dead_code)]
enum Register {
    /// First register of calibration data.
    /// Each register is 2 bytes long.
    DIG_T1 = 0x88,
    DIG_T2 = 0x8a,
    DIG_T3 = 0x8c,
    ID = 0xd0,
    RESET = 0xe0,
    /// measuring: [3]
    /// im_update: [0]
    STATUS = 0xf3,
    /// osrs_t: [7:5]
    /// osrs_p: [4:2]
    /// mode: [1:0]
    CTRL_MEAS = 0xf4,
    /// t_sb: [7:5]
    /// filter: [4:2]
    /// spi3w_en: [0]
    CONFIG = 0xf5,
    PRESS_MSB = 0xf7,
    PRESS_LSB = 0xf8,
    /// xlsb: [7:4]
    PRESS_XLSB = 0xf9,
    TEMP_MSB = 0xfa,
    TEMP_LSB = 0xfb,
    /// xlsb: [7:4]
    TEMP_XLSB = 0xfc,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct CalibrationData {
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
    // TODO: pressure calibration
}

/// CAUTION: calibration data puts least significant byte in the lowest address,
/// readouts do the opposite.
fn twobyte(lsb: u8, msb: u8) -> u16 {
    u16::from_be_bytes([msb, lsb])
}

impl CalibrationData {
    fn new(i2c_raw: &[u8]) -> Self {
        CalibrationData {
            dig_t1: twobyte(i2c_raw[0], i2c_raw[1]) as u16,
            dig_t2: twobyte(i2c_raw[2], i2c_raw[3]) as i16,
            dig_t3: twobyte(i2c_raw[4], i2c_raw[5]) as i16,
        }
    }

    fn temp_from_raw(&self, raw_temp: u32) -> i32 {
        let temp = raw_temp as i32; // guaranteed to succeed because raw temp has only 20 significant bits maximum.
        let dig_t1 = self.dig_t1 as i32; // same, 16-bits
        let dig_t2 = self.dig_t2 as i32; // same, 16-bits
        let dig_t3 = self.dig_t3 as i32; // same, 16-bits
                                         // From the datasheet
        let var1 = (((temp >> 3) - (dig_t1 << 1)) * dig_t2) >> 11;
        let a = (temp >> 4) - dig_t1;
        let var2 = (((a * a) >> 12) * dig_t3) >> 14;
        let t_fine = var1 + var2;
        ((t_fine * 5) + 128) >> 8
    }
}

/// Internal state.
/// Each state can lead to the next on in order of appearance.
#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    Uninitialized,
    InitId,
    /// It's not guaranteed that the MCU reset is the same as device power-on,
    /// so an explicit reset is necessary.
    InitResetting,
    InitWaitingReady,
    InitReadingCalibration,

    Idle(CalibrationData),

    // States related to sample readout
    /// One-shot mode request sent
    Configuring(CalibrationData),
    /// Sampling takes milliseconds, so spend most of that time sleeping.
    WaitingForAlarm(CalibrationData),
    /// Polling for the readout to become ready.
    Waiting(CalibrationData),
    /// Waiting for readout to return.
    /// This state can also lead back to Idle.
    Reading(CalibrationData),

    /// Reset cannot be attempted.
    /// This is because reset failed before, or because the ID is mismatched.
    IrrecoverableError,
    /// Irrecoverable. Currently only when init fails.
    /// Reset will clear this.
    Error,
    /// An unexpected, irrecoverable situation was encountered,
    /// and the driver is giving up.
    /// Reset clears this.
    Bug,
}

impl State {
    /// Changes state to one denoting this driver is buggy.
    fn to_bug(self) -> Self {
        match self {
            // A bug does not override the device not being present.
            State::IrrecoverableError => State::IrrecoverableError,
            _ => State::Bug,
        }
    }
}

/// Complies with the reading and writing protocol used by the sensor.
struct I2cWrapper<'a> {
    i2c: &'a dyn i2c::I2CDevice,
}

impl<'a> I2cWrapper<'a> {
    fn write<const COUNT: usize>(
        &self,
        buffer: &'static mut [u8],
        addr: Register,
        data: [u8; COUNT],
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        buffer[0] = addr as u8;
        buffer[1..][..COUNT].copy_from_slice(&data);
        self.i2c.enable();
        self.i2c.write(buffer, COUNT + 1)
    }

    /// Requests a read into buffer.
    /// Parse the result using `parse_read`.
    fn read(
        &self,
        buffer: &'static mut [u8],
        addr: Register,
        count: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        buffer[0] = addr as u8;
        self.i2c.enable();
        self.i2c.write_read(buffer, 1, count)
    }

    fn disable(&self) {
        self.i2c.disable()
    }

    fn parse_read(buffer: &[u8], count: u8) -> &[u8] {
        &buffer[..(count as usize)]
    }
}

pub struct Bmp280<'a, A: Alarm<'a>> {
    i2c: I2cWrapper<'a>,
    temperature_client: OptionalCell<&'a dyn hil::sensors::TemperatureClient>,
    // This might be better as a `RefCell`,
    // because `State` is multiple bytes due to the `CalibrationData`.
    // `Cell` requires Copy, which might get expensive, while `RefCell` doesn't.
    // It's probably not a good idea to split `CalibrationData`
    // into a separate place, because it will make state more duplicated.
    state: Cell<State>,
    /// Stores i2c commands
    buffer: TakeCell<'static, [u8]>,
    /// Needed to wait for readout completion, which can take milliseconds.
    /// It's possible to implement this without an alarm with busy polling, but that's wasteful.
    alarm: &'a A,
}

impl<'a, A: Alarm<'a>> Bmp280<'a, A> {
    pub fn new(i2c: &'a dyn i2c::I2CDevice, buffer: &'static mut [u8], alarm: &'a A) -> Self {
        Self {
            i2c: I2cWrapper { i2c },
            temperature_client: OptionalCell::empty(),
            state: Cell::new(State::Uninitialized),
            buffer: TakeCell::new(buffer),
            alarm: alarm,
        }
    }

    /// Resets the device and brings it into a known state.
    pub fn begin_reset(&self) -> Result<(), ErrorCode> {
        self.buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |buffer| match self.state.get() {
                State::Uninitialized | State::Error | State::Bug => {
                    let (ret, new_state) = match self.i2c.read(buffer, Register::ID, 1) {
                        Ok(()) => (Ok(()), State::InitId),
                        Err((_e, buffer)) => {
                            self.i2c.disable();
                            self.buffer.replace(buffer);
                            (Err(ErrorCode::FAIL), State::IrrecoverableError)
                        }
                    };
                    self.state.set(new_state);
                    ret
                }
                State::IrrecoverableError => Err(ErrorCode::NODEVICE),
                _ => Err(ErrorCode::ALREADY),
            })
    }

    pub fn read_temperature(&self) -> Result<(), ErrorCode> {
        match self.state.get() {
            // Actually, the sensor might be on, just in default state.
            State::Uninitialized => Err(ErrorCode::OFF),
            State::InitId
            | State::InitResetting
            | State::InitWaitingReady
            | State::InitReadingCalibration => Err(ErrorCode::BUSY),
            State::Idle(calibration) => {
                self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
                    // todo: use bitfield crate
                    // forced mode, oversampling 1
                    let val = 0b00100001;
                    let (ret, new_state) = match self.i2c.write(buffer, Register::CTRL_MEAS, [val])
                    {
                        Ok(()) => (Ok(()), State::Configuring(calibration)),
                        Err((_e, buffer)) => {
                            self.i2c.disable();
                            self.buffer.replace(buffer);
                            (Err(ErrorCode::FAIL), State::Idle(calibration))
                        }
                    };
                    self.state.set(new_state);
                    ret
                })
            }
            State::Configuring(_)
            | State::WaitingForAlarm(_)
            | State::Waiting(_)
            | State::Reading(_) => Err(ErrorCode::BUSY),
            State::Error | State::Bug => Err(ErrorCode::FAIL),
            State::IrrecoverableError => Err(ErrorCode::NODEVICE),
        }
    }

    fn handle_alarm(&self) {
        match self.state.get() {
            State::WaitingForAlarm(calibration) => self.buffer.take().map_or_else(
                || {
                    debug!("BMP280 No buffer available!");
                    self.state.set(State::IrrecoverableError)
                },
                |buffer| {
                    let new_state = match self.check_ready(buffer) {
                        Ok(()) => State::Waiting(calibration),
                        Err((_e, buffer)) => {
                            self.i2c.disable();
                            self.buffer.replace(buffer);
                            State::Idle(calibration)
                        }
                    };
                    self.state.set(new_state)
                },
            ),
            State::IrrecoverableError => {}
            other => {
                debug!("BMP280 received unexpected alarm in state {:?}", other);
                self.state.set(other.to_bug())
            }
        }
    }

    fn arm_alarm(&self) {
        // Datasheet says temp oversampling=1 makes a reading typically take 5.5ms.
        // (Maximally 6.4ms).
        let delay = self.alarm.ticks_from_us(6400);
        self.alarm.set_alarm(self.alarm.now(), delay);
    }

    fn check_ready(
        &self,
        buffer: &'static mut [u8],
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        self.i2c.read(buffer, Register::STATUS, 1)
    }
}

enum I2cOperation {
    Read {
        addr: Register,
        count: usize,
        fail_state: State,
    },
    Write {
        addr: Register,
        data: u8,
        fail_state: State,
    },
    Disable,
}

impl I2cOperation {
    fn check_ready(fail_state: State) -> Self {
        Self::Read {
            addr: Register::STATUS,
            count: 1,
            fail_state,
        }
    }
}

impl<'a, A: Alarm<'a>> i2c::I2CClient for Bmp280<'a, A> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        let mut temp_readout = None;
        let mut i2c_op = I2cOperation::Disable;

        let new_state = match status {
            Ok(()) => match self.state.get() {
                State::InitId => {
                    let id = I2cWrapper::parse_read(buffer, 1);
                    if id[0] == 0x58 {
                        i2c_op = I2cOperation::Write {
                            addr: Register::RESET,
                            data: 0xb6,
                            fail_state: State::IrrecoverableError,
                        };
                        State::InitResetting
                    } else {
                        State::IrrecoverableError
                    }
                }
                State::InitResetting => {
                    i2c_op = I2cOperation::check_ready(State::Error);
                    State::InitWaitingReady
                }
                State::InitWaitingReady => {
                    let waiting = I2cWrapper::parse_read(buffer, 1)[0];
                    if waiting & 0b1 == 0 {
                        // finished init
                        i2c_op = I2cOperation::Read {
                            addr: Register::DIG_T1,
                            count: 6,
                            fail_state: State::Error,
                        };
                        State::InitReadingCalibration
                    } else {
                        i2c_op = I2cOperation::check_ready(State::Error);
                        State::InitWaitingReady
                    }
                }
                State::InitReadingCalibration => {
                    let data = I2cWrapper::parse_read(buffer, 6);
                    let calibration = CalibrationData::new(data);
                    State::Idle(calibration)
                }
                // Readout-related states
                State::Configuring(calibration) => {
                    self.arm_alarm();
                    State::WaitingForAlarm(calibration)
                }
                State::Waiting(calibration) => {
                    let waiting_value = I2cWrapper::parse_read(buffer, 1);
                    // not waiting
                    if waiting_value[0] & 0b1000 == 0 {
                        i2c_op = I2cOperation::Read {
                            addr: Register::TEMP_MSB,
                            count: 3,
                            fail_state: State::Idle(calibration),
                        };
                        State::Reading(calibration)
                    } else {
                        i2c_op = I2cOperation::check_ready(State::Idle(calibration));
                        State::Waiting(calibration)
                    }
                }
                State::Reading(calibration) => {
                    let readout = I2cWrapper::parse_read(buffer, 3);
                    let msb = readout[0] as u32;
                    let lsb = readout[1] as u32;
                    let xlsb = readout[2] as u32;
                    let raw_temp = (msb << 12) + (lsb << 4) + (xlsb >> 4);
                    temp_readout = Some(Ok(calibration.temp_from_raw(raw_temp)));
                    State::Idle(calibration)
                }
                other => {
                    debug!("BMP280 received unexpected i2c reply in state {:?}", other);
                    other.to_bug()
                }
            },
            Err(i2c_err) => match self.state.get() {
                State::Configuring(calibration)
                | State::Waiting(calibration)
                | State::Reading(calibration) => {
                    temp_readout = Some(Err(i2c_err.into()));
                    State::Idle(calibration)
                }
                State::InitId
                | State::InitResetting
                | State::InitWaitingReady
                | State::InitReadingCalibration => State::Error,
                other => {
                    debug!("BMP280 received unexpected i2c reply in state {:?}", other);
                    other.to_bug()
                }
            },
        };

        // Try enqueueing the requested i2c operation
        let new_state = match i2c_op {
            I2cOperation::Disable => {
                self.i2c.disable();
                self.buffer.replace(buffer);
                new_state
            }
            I2cOperation::Read {
                addr,
                count,
                fail_state,
            } => {
                if let Err((_e, buffer)) = self.i2c.read(buffer, addr, count) {
                    self.i2c.disable();
                    self.buffer.replace(buffer);
                    fail_state
                } else {
                    new_state
                }
            }
            I2cOperation::Write {
                addr,
                data,
                fail_state,
            } => {
                if let Err((_e, buffer)) = self.i2c.write(buffer, addr, [data]) {
                    self.i2c.disable();
                    self.buffer.replace(buffer);
                    fail_state
                } else {
                    new_state
                }
            }
        };

        // Setting state before the callback,
        // in case the callback wants to use the same driver again.
        self.state.set(new_state);
        if let Some(temp) = temp_readout {
            self.temperature_client.map(|cb| cb.callback(temp));
        }
    }
}

impl<'a, A: Alarm<'a>> hil::sensors::TemperatureDriver<'a> for Bmp280<'a, A> {
    fn set_client(&self, client: &'a dyn hil::sensors::TemperatureClient) {
        self.temperature_client.set(client)
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        self.read_temperature()
    }
}

impl<'a, A: hil::time::Alarm<'a>> hil::time::AlarmClient for Bmp280<'a, A> {
    fn alarm(&self) {
        self.handle_alarm()
    }
}
