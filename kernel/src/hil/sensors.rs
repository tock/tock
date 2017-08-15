//! Interfaces for environment sensors

use returncode::ReturnCode;

/// A basic interface for a temperature sensor
pub trait TemperatureDriver {
    fn set_client(&self, client: &'static TemperatureClient);
    fn read_ambient_temperature(&self) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }
    fn read_cpu_temperature(&self) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }
}

/// Client for receiving temperature readings.
pub trait TemperatureClient {
    /// Called when a temperature reading has completed.
    ///
    /// - `value`: the most recently read temperature in hundredths of degrees
    /// centigrate.
    fn callback(&self, value: usize);
}

/// A basic interface for a humidity sensor
pub trait HumidityDriver {
    fn set_client(&self, client: &'static HumidityClient);
    fn read_humidity(&self) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }
}

/// Client for receiving temperature readings.
pub trait HumidityClient {
    /// Called when a humidity reading has completed.
    ///
    /// - `value`: the most recently read temperature in hundredths of percent.
    fn callback(&self, value: usize);
}
