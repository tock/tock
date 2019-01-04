//! Driver for the TI TMP006 infrared thermopile contactless temperature sensor.
//!
//! <http://www.ti.com/product/TMP006>
//!
//! > The TMP006 and TMP006B are fully integrated MEMs thermopile sensors that
//! > measure the temperature of an object without having to be in direct
//! > contact. The thermopile absorbs passive infrared energy from an object at
//! > wavelengths between 4 um to 16 um within the end-user defined field of
//! > view.

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::math::{get_errno, sqrtf32};
use kernel::hil::gpio::{Client, InterruptMode, Pin};
use kernel::hil::i2c;
use kernel::{AppId, Callback, Driver, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::TMP006 as usize;

pub static mut BUFFER: [u8; 3] = [0; 3];

const MAX_SAMPLING_RATE: u8 = 0x0;
const DEFAULT_SAMPLING_RATE: u8 = 0x02;

// temperature calculation constants
//  From TMP006 User's Guide section 5.1
//  S_0 should be determined from calibration and ranges from 5E-14 to 7E-14
//  We have selected 5E-14 experimentally
const S_0: f32 = 5E-14;
const A_1: f32 = 1.75E-3;
const A_2: f32 = -1.678E-5;
const T_REF: f32 = 298.15;
const B_0: f32 = -2.94E-5;
const B_1: f32 = -5.7E-7;
const B_2: f32 = 4.63E-9;
const C_2: f32 = 13.4;
const K_TO_C: f32 = -273.15;
const C_TO_K: f32 = 273.15;
const NV_TO_V: f32 = 1E9;
const T_DIE_CONVERT: f32 = 0.03125;
const V_OBJ_CONVERT: f32 = 156.25;

#[allow(dead_code)]
enum Registers {
    SensorVoltage = 0x00,
    DieTemperature = 0x01,
    Configuration = 0x02,
    ManufacturerID = 0xFE,
    DeviceID = 0xFF,
}

type SensorVoltage = i16;

/// States of the I2C protocol with the TMP006. There are three sequences:
///
/// ### Enable sensor
///
/// Configure -> ()
///
/// ### Disable sensor
///
/// Disconfigure -> ()
///
/// ### Read temperature
///
/// SetRegSensorVoltage -->
///     ReadingSensorVoltage --(voltage)->
///         SetRegDieTemperature(voltage) --(voltage)->
///             ReadingDieTemperature --(unless repeated_mode)->
///                 Disconfigure
#[derive(Clone, Copy, PartialEq)]
enum ProtocolState {
    Idle,

    /// Enable sensor by setting the configuration register.
    Configure,

    /// Disable sensor by setting the configuration register. Optionally contains the most recent
    /// temperature to give back to callbacks.
    Deconfigure(Option<f32>),

    /// Set the active register to sensor voltage.
    SetRegSensorVoltage,

    /// Read the sensor voltage register.
    ReadingSensorVoltage,

    /// Set the active register to die temperature, carrying over the sensor
    /// voltage reading.
    SetRegDieTemperature(SensorVoltage),

    /// Read the die temperature register, carrying over the sensor voltage
    /// reading.
    ReadingDieTemperature(SensorVoltage),
}

pub struct TMP006<'a> {
    i2c: &'a i2c::I2CDevice,
    interrupt_pin: &'a Pin,
    sampling_period: Cell<u8>,
    repeated_mode: Cell<bool>,
    callback: OptionalCell<Callback>,
    protocol_state: Cell<ProtocolState>,
    buffer: TakeCell<'static, [u8]>,
}

impl TMP006<'a> {
    /// The `interrupt_pin` must be pulled-up since the TMP006 is open-drain.
    pub fn new(
        i2c: &'a i2c::I2CDevice,
        interrupt_pin: &'a Pin,
        buffer: &'static mut [u8],
    ) -> TMP006<'a> {
        // setup and return struct
        TMP006 {
            i2c: i2c,
            interrupt_pin: interrupt_pin,
            sampling_period: Cell::new(DEFAULT_SAMPLING_RATE),
            repeated_mode: Cell::new(false),
            callback: OptionalCell::empty(),
            protocol_state: Cell::new(ProtocolState::Idle),
            buffer: TakeCell::new(buffer),
        }
    }

    fn enable_sensor(&self, sampling_period: u8) {
        // enable and configure TMP006
        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            self.i2c.enable();

            let config = 0x7100 | (((sampling_period & 0x7) as u16) << 9);
            buf[0] = Registers::Configuration as u8;
            buf[1] = ((config & 0xFF00) >> 8) as u8;
            buf[2] = (config & 0x00FF) as u8;
            self.i2c.write(buf, 3);
            self.protocol_state.set(ProtocolState::Configure);
        });
    }

    fn disable_sensor(&self, temperature: Option<f32>) {
        // disable the TMP006
        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            self.i2c.enable();

            let config = 0x0000;
            buf[0] = Registers::Configuration as u8;
            buf[1] = ((config & 0xFF00) >> 8) as u8;
            buf[2] = (config & 0x00FF) as u8;
            self.i2c.write(buf, 3);
            self.protocol_state
                .set(ProtocolState::Deconfigure(temperature));
        });
    }

    fn enable_interrupts(&self) {
        // setup interrupts from the sensor
        self.interrupt_pin.make_input();
        self.interrupt_pin
            .enable_interrupt(0, InterruptMode::FallingEdge);
    }

    fn disable_interrupts(&self) {
        // disable interrupts from the sensor
        self.interrupt_pin.disable_interrupt();
        self.interrupt_pin.disable();
    }
}

fn calculate_temperature(sensor_voltage: i16, die_temperature: i16) -> f32 {
    // do calculation of actual temperature
    //  Calculations based on TMP006 User's Guide section 5.1
    let t_die = ((die_temperature >> 2) as f32) * T_DIE_CONVERT + C_TO_K;
    let t_adj = t_die - T_REF;
    let s = S_0 * (1.0 + A_1 * t_adj + A_2 * t_adj * t_adj);

    let v_obj = (sensor_voltage as f32) * V_OBJ_CONVERT / NV_TO_V;
    let v_os = B_0 + B_1 * t_adj + B_2 * t_adj * t_adj;

    let v_adj = v_obj - v_os;
    let f_v_obj = v_adj + C_2 * v_adj * v_adj;

    let t_kelvin = sqrtf32(sqrtf32(t_die * t_die * t_die * t_die + (f_v_obj / s)));
    let t_celsius = t_kelvin + K_TO_C;

    // return data value
    t_celsius
}

impl i2c::I2CClient for TMP006<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        // TODO(alevy): handle protocol errors
        match self.protocol_state.get() {
            ProtocolState::Configure => {
                self.buffer.replace(buffer);
                self.enable_interrupts();
                self.i2c.disable();
                self.protocol_state.set(ProtocolState::Idle);
            }
            ProtocolState::Deconfigure(temperature) => {
                self.buffer.replace(buffer);
                self.disable_interrupts();
                self.i2c.disable();
                self.protocol_state.set(ProtocolState::Idle);
                temperature.map(|temp_val| {
                    self.callback
                        .take()
                        .map(|mut cb| cb.schedule(temp_val as usize, get_errno() as usize, 0));
                });
            }
            ProtocolState::SetRegSensorVoltage => {
                // Read sensor voltage register
                self.i2c.read(buffer, 2);
                self.protocol_state.set(ProtocolState::ReadingSensorVoltage);
            }
            ProtocolState::ReadingSensorVoltage => {
                let sensor_voltage = (((buffer[0] as u16) << 8) | buffer[1] as u16) as i16;

                // Select die temperature register
                buffer[0] = Registers::DieTemperature as u8;
                self.i2c.write(buffer, 1);

                self.protocol_state
                    .set(ProtocolState::SetRegDieTemperature(sensor_voltage));
            }
            ProtocolState::SetRegDieTemperature(sensor_voltage) => {
                // Read die temperature register
                self.i2c.read(buffer, 2);
                self.protocol_state
                    .set(ProtocolState::ReadingDieTemperature(sensor_voltage));
            }
            ProtocolState::ReadingDieTemperature(sensor_voltage) => {
                let die_temperature = (((buffer[0] as u16) << 8) | buffer[1] as u16) as i16;
                self.buffer.replace(buffer);

                let temp_val = calculate_temperature(sensor_voltage, die_temperature);

                // disable callback and sensing if in single-shot mode
                if self.repeated_mode.get() == false {
                    // disable temperature sensor. When disabling is finished, we will give the
                    // temperature to the callback.
                    self.disable_sensor(Some(temp_val));
                } else {
                    // send value to callback
                    self.callback
                        .map(|cb| cb.schedule(temp_val as usize, get_errno() as usize, 0));

                    self.i2c.disable();
                }
            }
            _ => {}
        }
    }
}

impl Client for TMP006<'a> {
    fn fired(&self, _: usize) {
        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            self.i2c.enable();

            // select sensor voltage register and read it
            buf[0] = Registers::SensorVoltage as u8;
            self.i2c.write(buf, 1);
            self.protocol_state.set(ProtocolState::SetRegSensorVoltage);
        });
    }
}

impl Driver for TMP006<'a> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // single temperature reading with callback
            0 => {
                // single sample mode
                self.repeated_mode.set(false);

                // set callback function
                self.callback.insert(callback);

                // enable sensor
                //  turn up the sampling rate so we get the sample faster
                self.enable_sensor(MAX_SAMPLING_RATE);

                ReturnCode::SUCCESS
            }

            // periodic temperature reading subscription
            1 => {
                // periodic sampling mode
                self.repeated_mode.set(true);

                // set callback function
                self.callback.insert(callback);

                // enable temperature sensor
                self.enable_sensor(self.sampling_period.get());

                ReturnCode::SUCCESS
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            // set period for sensing
            1 => {
                // bounds check on the period
                if (data & 0xFFFFFFF8) != 0 {
                    return ReturnCode::EINVAL;
                }

                // set period value
                self.sampling_period.set((data & 0x7) as u8);

                ReturnCode::SUCCESS
            }

            // unsubscribe callback
            2 => {
                // clear callback function
                self.callback.clear();

                // disable temperature sensor
                self.disable_sensor(None);

                ReturnCode::SUCCESS
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
