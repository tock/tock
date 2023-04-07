// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

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
use kernel::ErrorCode;

struct SensorTestCallback {
    temperature_done: Cell<bool>,
    humidity_done: Cell<bool>,
    co2_done: Cell<bool>,
    tvoc_done: Cell<bool>,
    calibration_temp: Cell<Option<i32>>,
    calibration_humidity: Cell<Option<u32>>,
}

unsafe impl Sync for SensorTestCallback {}

impl<'a> SensorTestCallback {
    const fn new() -> Self {
        SensorTestCallback {
            temperature_done: Cell::new(false),
            humidity_done: Cell::new(false),
            co2_done: Cell::new(false),
            tvoc_done: Cell::new(false),
            calibration_temp: Cell::new(None),
            calibration_humidity: Cell::new(None),
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
    fn callback(&self, result: Result<i32, ErrorCode>) {
        self.temperature_done.set(true);
        self.calibration_temp.set(Some(result.unwrap()));

        debug!("Temperature: {}", result.unwrap());
    }
}

impl<'a> HumidityClient for SensorTestCallback {
    fn callback(&self, value: usize) {
        self.humidity_done.set(true);
        self.calibration_humidity.set(Some(value as u32));

        debug!("Humidity: {}", value);
    }
}

impl<'a> AirQualityClient for SensorTestCallback {
    fn environment_specified(&self, result: Result<(), ErrorCode>) {
        result.unwrap();
    }

    fn co2_data_available(&self, value: Result<u32, ErrorCode>) {
        self.co2_done.set(true);

        debug!("CO2: {} ppm", value.unwrap());
    }

    fn tvoc_data_available(&self, value: Result<u32, ErrorCode>) {
        self.tvoc_done.set(true);

        debug!("TVOC: {} ppb", value.unwrap());
    }
}

static CALLBACK: SensorTestCallback = SensorTestCallback::new();

#[test_case]
fn run_bme280_temperature() {
    debug!("check run BME280 Temperature... ");
    run_kernel_op(100);

    let bme280 = unsafe { BME280.unwrap() };

    // Make sure the device is ready for us.
    // The setup can take a little bit of time
    run_kernel_op(800000);

    TemperatureDriver::set_client(bme280, &CALLBACK);
    CALLBACK.reset();

    bme280.read_temperature().unwrap();

    run_kernel_op(50000);
    assert_eq!(CALLBACK.temperature_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn run_bme280_humidity() {
    debug!("check run BME280 Humidity... ");
    run_kernel_op(100);

    let bme280 = unsafe { BME280.unwrap() };

    // Make sure the debice is ready for us.
    // The setup can take a little bit of time
    run_kernel_op(800000);

    HumidityDriver::set_client(bme280, &CALLBACK);
    CALLBACK.reset();

    bme280.read_humidity().unwrap();

    run_kernel_op(50000);
    assert_eq!(CALLBACK.humidity_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn run_ccs811_co2() {
    debug!("check run CCS811 CO2... ");
    run_kernel_op(100);

    let ccs811 = unsafe { CCS811.unwrap() };

    // Make sure the device is ready for us.
    // The setup can take a little bit of time
    run_kernel_op(800000);

    AirQualityDriver::set_client(ccs811, &CALLBACK);
    CALLBACK.reset();

    ccs811
        .specify_environment(
            CALLBACK.calibration_temp.get(),
            CALLBACK.calibration_humidity.get(),
        )
        .unwrap();

    run_kernel_op(7000);

    ccs811.read_co2().unwrap();

    run_kernel_op(7000);
    assert_eq!(CALLBACK.co2_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn run_ccs811_tvoc() {
    debug!("check run CCS811 TVOC... ");
    run_kernel_op(100);

    let ccs811 = unsafe { CCS811.unwrap() };

    // Make sure the device is ready for us.
    // The setup can take a little bit of time
    run_kernel_op(800000);

    AirQualityDriver::set_client(ccs811, &CALLBACK);
    CALLBACK.reset();

    ccs811
        .specify_environment(
            CALLBACK.calibration_temp.get(),
            CALLBACK.calibration_humidity.get(),
        )
        .unwrap();

    run_kernel_op(7000);

    ccs811.read_tvoc().unwrap();

    run_kernel_op(7000);
    assert_eq!(CALLBACK.tvoc_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}
