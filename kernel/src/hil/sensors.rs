// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interfaces for environment sensors

use crate::errorcode::ErrorCode;

/// A basic interface for a temperature sensor
pub trait TemperatureDriver<'a> {
    fn set_client(&self, client: &'a dyn TemperatureClient);
    fn read_temperature(&self) -> Result<(), ErrorCode>;
}

/// Client for receiving temperature readings.
pub trait TemperatureClient {
    /// Called when a temperature reading has completed.
    ///
    /// - `value`: the most recently read temperature in hundredths of degrees
    /// centigrade (centiCelsius), or Err on failure.
    fn callback(&self, value: Result<i32, ErrorCode>);
}

/// A basic interface for a humidity sensor
pub trait HumidityDriver<'a> {
    fn set_client(&self, client: &'a dyn HumidityClient);
    fn read_humidity(&self) -> Result<(), ErrorCode>;
}

/// Client for receiving humidity readings.
pub trait HumidityClient {
    /// Called when a humidity reading has completed.
    ///
    /// - `value`: the most recently read humidity in hundredths of percent.
    fn callback(&self, value: usize);
}

/// A basic interface for a moisture sensor
pub trait MoistureDriver<'a> {
    fn set_client(&self, client: &'a dyn MoistureClient);

    /// Read the moisture value from a sensor. The value is returned
    /// via the `MoistureClient` callback.
    ///
    /// This function might return the following errors:
    /// - `BUSY`: Indicates that the hardware is busy with an existing operation
    ///   or initialisation/calibration.
    /// - `NOSUPPORT`: Indicates that this data type isn't supported.
    fn read_moisture(&self) -> Result<(), ErrorCode>;
}

/// Client for receiving moisture readings.
pub trait MoistureClient {
    /// Called when a moisture reading has completed.
    ///
    /// - `value`: the most recently read moisture in hundredths of
    /// percent, or Err on failure.
    ///
    /// This function might return the following errors:
    /// - `BUSY`: Indicates that the hardware is busy with an existing operation
    ///   or initialisation/calibration.
    /// - `NOSUPPORT`: Indicates that this data type isn't supported.
    fn callback(&self, value: Result<usize, ErrorCode>);
}

/// A basic interface for a Air Quality sensor
pub trait AirQualityDriver<'a> {
    /// Set the client to be notified when the capsule has data ready.
    fn set_client(&self, client: &'a dyn AirQualityClient);

    /// Specify the temperature and humidity used in calculating the air
    /// quality.
    ///
    /// The temperature is specified in degrees Celsius and the humidity
    /// is specified as a percentage.
    ///
    /// This is an optional call and doesn't have to be used, but on most
    /// hardware can be used to improve the measurement accuracy.
    ///
    /// This function might return the following errors:
    /// - `BUSY`: Indicates that the hardware is busy with an existing operation
    ///   or initialisation/calibration.
    /// - `NOSUPPORT`: Indicates that this data type isn't supported.
    fn specify_environment(
        &self,
        temp: Option<i32>,
        humidity: Option<u32>,
    ) -> Result<(), ErrorCode>;

    /// Read the CO2 or equivalent CO2 (eCO2) from the sensor.
    /// This will trigger the `AirQualityClient` `co2_data_available()`
    /// callback when the data is ready.
    ///
    /// This function might return the following errors:
    /// - `BUSY`: Indicates that the hardware is busy with an existing operation
    ///   or initialisation/calibration.
    /// - `NOSUPPORT`: Indicates that this data type isn't supported.
    fn read_co2(&self) -> Result<(), ErrorCode>;

    /// Read the Total Organic Compound (TVOC) from the sensor.
    /// This will trigger the `AirQualityClient` `tvoc_data_available()`
    /// callback when the data is ready.
    ///
    /// This function might return the following errors:
    /// - `BUSY`: Indicates that the hardware is busy with an existing operation
    ///   or initialisation/calibration.
    /// - `NOSUPPORT`: Indicates that this data type isn't supported.
    fn read_tvoc(&self) -> Result<(), ErrorCode>;
}

/// Client for receiving Air Quality readings
pub trait AirQualityClient {
    /// Called when the environment specify command has completed.
    fn environment_specified(&self, result: Result<(), ErrorCode>);

    /// Called when a CO2 or equivalent CO2 (eCO2) reading has completed.
    ///
    /// - `value`: will contain the latest CO2 reading in ppm. An example value
    ///   might be `400`.
    fn co2_data_available(&self, value: Result<u32, ErrorCode>);

    /// Called when a Total Organic Compound (TVOC) reading has completed.
    ///
    /// - `value`: will contain the latest TVOC reading in ppb. An example value
    ///   might be `0`.
    fn tvoc_data_available(&self, value: Result<u32, ErrorCode>);
}

/// A basic interface for a proximity sensor
pub trait ProximityDriver<'a> {
    fn set_client(&self, client: &'a dyn ProximityClient);
    /// Callback issued after sensor reads proximity value
    fn read_proximity(&self) -> Result<(), ErrorCode>;
    /// Callback issued after sensor reads proximity value greater than 'high_threshold' or less than 'low_threshold'
    ///
    /// To elaborate, the callback is not issued by the driver until (prox_reading >= high_threshold || prox_reading <= low_threshold).
    /// When (prox_reading >= high_threshold || prox_reading <= low_threshold) is read by the sensor, an I2C interrupt is generated and sent to the kernel
    /// which prompts the driver to collect the proximity reading from the sensor and perform the callback.
    /// Any apps issuing this command will have to wait for the proximity reading to fall within the aforementioned ranges in order to received a callback.
    /// Threshold: A value of range [0 , 255] which represents at what proximity reading ranges an interrupt will occur.
    fn read_proximity_on_interrupt(
        &self,
        low_threshold: u8,
        high_threshold: u8,
    ) -> Result<(), ErrorCode>;
}

pub trait ProximityClient {
    /// Called when a proximity reading has completed.
    ///
    /// - `value`: the most recently read proximity value which ranges [0 , 255]...
    /// where 255 -> object is closest readable distance, 0 -> object is farthest readable distance.
    fn callback(&self, value: u8);
}

/// A basic interface for an ambient light sensor.
pub trait AmbientLight<'a> {
    /// Set the client to be notified when the capsule has data ready or has
    /// finished some command.  This is likely called in a board's `main.rs`.
    fn set_client(&self, client: &'a dyn AmbientLightClient);

    /// Get a single instantaneous reading of the ambient light intensity.
    fn read_light_intensity(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NODEVICE)
    }
}

/// Client for receiving light intensity readings.
pub trait AmbientLightClient {
    /// Called when an ambient light reading has completed.
    ///
    /// - `lux`: the most recently read ambient light reading in lux (lx).
    fn callback(&self, lux: usize);
}

/// A basic interface for a 9-DOF compatible chip.
///
/// This trait provides a standard interface for chips that implement
/// some or all of a nine degrees of freedom (accelerometer, magnetometer,
/// gyroscope) sensor. Any interface functions that a chip cannot implement
/// can be ignored by the chip capsule and an error will automatically be
/// returned.
pub trait NineDof<'a> {
    /// Set the client to be notified when the capsule has data ready or
    /// has finished some command. This is likely called in a board's main.rs
    /// and is set to the virtual_ninedof.rs driver.
    fn set_client(&self, client: &'a dyn NineDofClient);

    /// Get a single instantaneous reading of the acceleration in the
    /// X,Y,Z directions.
    fn read_accelerometer(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NODEVICE)
    }

    /// Get a single instantaneous reading from the magnetometer in all
    /// three directions.
    fn read_magnetometer(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NODEVICE)
    }

    /// Get a single instantaneous reading from the gyroscope of the rotation
    /// around all three axes.
    fn read_gyroscope(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NODEVICE)
    }
}

/// Client for receiving done events from the chip.
pub trait NineDofClient {
    /// Signals a command has finished. The arguments will most likely be passed
    /// over the syscall interface to an application.
    fn callback(&self, arg1: usize, arg2: usize, arg3: usize);
}

/// Basic Interface for Sound Pressure
pub trait SoundPressure<'a> {
    /// Read the sound pressure level
    fn read_sound_pressure(&self) -> Result<(), ErrorCode>;

    /// Enable
    ///
    /// As this is usually a microphone, some boards require an explicit enable
    /// so that they can turn on an LED. This function enables that microphone and LED.
    /// Not calling this function may result in inaccurate readings.
    fn enable(&self) -> Result<(), ErrorCode>;

    /// Disable
    ///
    /// As this is usually a microphone, some boards require an explicit enable
    /// so that they can turn on an LED. This function turns off that microphone. Readings
    /// performed after this function call might return inaccurate.
    fn disable(&self) -> Result<(), ErrorCode>;

    /// Set the client
    fn set_client(&self, client: &'a dyn SoundPressureClient);
}

pub trait SoundPressureClient {
    /// Signals the sound pressure in dB
    fn callback(&self, ret: Result<(), ErrorCode>, sound_pressure: u8);
}

/// A Basic interface for a barometer sensor.
pub trait PressureDriver<'a> {
    /// Used to initialize a atmospheric pressure reading
    ///
    /// This function might return the following errors:
    /// - `BUSY`: Indicates that the hardware is busy with an existing operation
    ///   or initialisation/calibration.
    /// - `FAIL`: Failed to correctly communicate over communication protocol.
    /// - `NOSUPPORT`: Indicates that this data type isn't supported.
    fn read_atmospheric_pressure(&self) -> Result<(), ErrorCode>;

    /// Set the client
    fn set_client(&self, client: &'a dyn PressureClient);
}

pub trait PressureClient {
    /// Called when a atmospheric pressure reading has completed.
    ///
    /// Returns the value in hPa.
    fn callback(&self, pressure: Result<u32, ErrorCode>);
}

/// A basic interface for distance sensor.
pub trait Distance<'a> {
    /// Set the client
    fn set_client(&self, client: &'a dyn DistanceClient);

    /// Initiates a distance reading from the sensor.
    ///
    /// This function might return the following errors:
    /// - `BUSY`: Indicates that the hardware is currently busy.
    /// - `FAIL`: Indicates that there was a failure in communication.
    fn read_distance(&self) -> Result<(), ErrorCode>;

    /// Get the maximum distance the sensor can measure in mm
    fn get_maximum_distance(&self) -> u32;

    /// Get the minimum distance the sensor can measure in mm
    fn get_minimum_distance(&self) -> u32;
}

/// Client for receiving distance readings.
pub trait DistanceClient {
    /// Called when a distance measurement has completed.
    ///
    /// - `distance`: the most recently measured distance in millimeters. If
    ///   there was an error, this will be `Err(ErrorCode)`.
    fn callback(&self, distance: Result<u32, ErrorCode>);
}

/// A basic interface for a rain fall sensor
pub trait RainFallDriver<'a> {
    fn set_client(&self, client: &'a dyn RainFallClient);

    /// Read the rain fall value from a sensor. The value is returned
    /// via the `RainFallClient` callback.
    ///
    /// - `hours`: the number of hours of rainfall to report. 1 to 24 hours are
    ///   valid values (if supported by the hardware).
    ///
    /// This function might return the following errors:
    /// - `BUSY`: Indicates that the hardware is busy with an existing operation
    ///   or initialisation/calibration.
    /// - `NOSUPPORT`: Indicates that the value of `hours` is not supported.
    fn read_rainfall(&self, hours: usize) -> Result<(), ErrorCode>;
}

/// Client for receiving moisture readings.
pub trait RainFallClient {
    /// Called when a moisture reading has completed.
    ///
    /// - `value`: the number of um of rain in the time period specified, or Err
    ///   on failure.
    ///
    /// This function might return the following errors:
    /// - `BUSY`: Indicates that the hardware is busy with an existing operation
    ///   or initialisation/calibration.
    /// - `NOSUPPORT`: Indicates that the value of `hours` is not supported.
    fn callback(&self, value: Result<usize, ErrorCode>);
}
