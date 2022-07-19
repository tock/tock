//! Test that we can get temperature, humidity and pressure from the BME280
//! chip.
//! This test requires the "SparkFun Environmental Combo Breakout" board
//! connected via the Qwiic connector
//! (https://www.sparkfun.com/products/14348).

use crate::tests::run_kernel_op;
use crate::BME280;
use core::cell::Cell;
use kernel::debug;
use kernel::hil::sensors::{HumidityClient, HumidityDriver, TemperatureClient, TemperatureDriver};
use kernel::static_init;

struct SensorTestCallback {
    temperature_done: Cell<bool>,
    humidity_done: Cell<bool>,
}

unsafe impl Sync for SensorTestCallback {}

impl<'a> SensorTestCallback {
    fn new() -> Self {
        SensorTestCallback {
            temperature_done: Cell::new(false),
            humidity_done: Cell::new(false),
        }
    }

    fn reset(&self) {
        self.temperature_done.set(false);
        self.humidity_done.set(false);
    }
}

impl<'a> TemperatureClient for SensorTestCallback {
    fn callback(&self, value: usize) {
        self.temperature_done.set(true);

        debug!("Temperature: {}", value);
    }
}

impl<'a> HumidityClient for SensorTestCallback {
    fn callback(&self, value: usize) {
        self.humidity_done.set(true);

        debug!("Humidity: {}", value);
    }
}

#[test_case]
fn run_bme280_temperature() {
    debug!("check run BME280 Temperature... ");
    run_kernel_op(100);

    let bme280 = unsafe { BME280.unwrap() };
    let callback = unsafe { static_init!(SensorTestCallback, SensorTestCallback::new()) };

    // Make sure the device is ready for us.
    // The setup can take a little bit of time
    run_kernel_op(400000);

    TemperatureDriver::set_client(bme280, callback);
    callback.reset();

    bme280.read_temperature().unwrap();

    run_kernel_op(50000);
    assert_eq!(callback.temperature_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn run_humidity() {
    debug!("check run BME280 Humidity... ");
    run_kernel_op(100);

    let bme280 = unsafe { BME280.unwrap() };
    let callback = unsafe { static_init!(SensorTestCallback, SensorTestCallback::new()) };

    // Make sure the debice is ready for us.
    // The setup can take a little bit of time
    run_kernel_op(400000);

    HumidityDriver::set_client(bme280, callback);
    callback.reset();

    bme280.read_humidity().unwrap();

    run_kernel_op(50000);
    assert_eq!(callback.humidity_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}
