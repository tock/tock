//! Test that we can get temperature, humidity and pressure from the BME280
//! chip.
//! This test requires the "SparkFun Environmental Combo Breakout" board
//! connected via the Qwiic connector
//! (https://www.sparkfun.com/products/14348).

use crate::tests::run_kernel_op;
use crate::{BME280, CCS811};
use core::cell::Cell;
use kernel::debug;
use kernel::hil::sensors::{
    AirQualityClient, AirQualityDriver, HumidityClient, HumidityDriver, TemperatureClient,
    TemperatureDriver,
};
use kernel::static_init;
use kernel::ErrorCode;

struct SensorTestCallback {
    temperature_done: Cell<bool>,
    humidity_done: Cell<bool>,
    co2_done: Cell<bool>,
    tvoc_done: Cell<bool>,
}

unsafe impl Sync for SensorTestCallback {}

impl<'a> SensorTestCallback {
    fn new() -> Self {
        SensorTestCallback {
            temperature_done: Cell::new(false),
            humidity_done: Cell::new(false),
            co2_done: Cell::new(false),
            tvoc_done: Cell::new(false),
        }
    }

    fn reset(&self) {
        self.temperature_done.set(false);
        self.humidity_done.set(false);
        self.co2_done.set(false);
        self.tvoc_done.set(false);
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

impl<'a> AirQualityClient for SensorTestCallback {
    fn co2_data_available(&self, value: Result<u32, ErrorCode>) {
        self.co2_done.set(true);

        debug!("CO2: {} ppm", value.unwrap());
    }

    fn tvoc_data_available(&self, value: Result<u32, ErrorCode>) {
        self.tvoc_done.set(true);

        debug!("TVOC: {} ppb", value.unwrap());
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
    run_kernel_op(800000);

    TemperatureDriver::set_client(bme280, callback);
    callback.reset();

    bme280.read_temperature().unwrap();

    run_kernel_op(50000);
    assert_eq!(callback.temperature_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn run_bme280_humidity() {
    debug!("check run BME280 Humidity... ");
    run_kernel_op(100);

    let bme280 = unsafe { BME280.unwrap() };
    let callback = unsafe { static_init!(SensorTestCallback, SensorTestCallback::new()) };

    // Make sure the debice is ready for us.
    // The setup can take a little bit of time
    run_kernel_op(800000);

    HumidityDriver::set_client(bme280, callback);
    callback.reset();

    bme280.read_humidity().unwrap();

    run_kernel_op(50000);
    assert_eq!(callback.humidity_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn run_ccs811_co2() {
    debug!("check run CCS811 CO2... ");
    run_kernel_op(100);

    let ccs811 = unsafe { CCS811.unwrap() };
    let callback = unsafe { static_init!(SensorTestCallback, SensorTestCallback::new()) };

    // Make sure the device is ready for us.
    // The setup can take a little bit of time
    run_kernel_op(800000);

    AirQualityDriver::set_client(ccs811, callback);
    callback.reset();

    ccs811.read_co2().unwrap();

    run_kernel_op(7000);
    assert_eq!(callback.co2_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn run_ccs811_tvoc() {
    debug!("check run CCS811 TVOC... ");
    run_kernel_op(100);

    let ccs811 = unsafe { CCS811.unwrap() };
    let callback = unsafe { static_init!(SensorTestCallback, SensorTestCallback::new()) };

    // Make sure the device is ready for us.
    // The setup can take a little bit of time
    run_kernel_op(800000);

    AirQualityDriver::set_client(ccs811, callback);
    callback.reset();

    ccs811.read_tvoc().unwrap();

    run_kernel_op(7000);
    assert_eq!(callback.tvoc_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}
