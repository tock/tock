// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! SyscallDriver for RP2040 ADC MCU temperature sensor

use core::cell::Cell;
use kernel::hil::adc;
use kernel::hil::sensors;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Temperature as usize;

#[derive(Copy, Clone, PartialEq)]
pub enum Status {
    Read,
    Idle,
}

pub struct TemperatureRp2040<'a, A: adc::AdcChannel<'a>> {
    adc: &'a A,
    slope: f32,
    v_27: f32,
    temperature_client: OptionalCell<&'a dyn sensors::TemperatureClient>,
    status: Cell<Status>,
}

impl<'a, A: adc::AdcChannel<'a>> TemperatureRp2040<'a, A> {
    /// slope - device specific slope found in datasheet
    /// v_27 - voltage at 27 degrees Celsius found in datasheet
    pub fn new(adc: &'a A, slope: f32, v_27: f32) -> TemperatureRp2040<'a, A> {
        TemperatureRp2040 {
            adc: adc,
            slope: slope,
            v_27: v_27,
            temperature_client: OptionalCell::empty(),
            status: Cell::new(Status::Idle),
        }
    }
}

impl<'a, A: adc::AdcChannel<'a>> adc::Client for TemperatureRp2040<'a, A> {
    fn sample_ready(&self, sample: u16) {
        self.status.set(Status::Idle);
        self.temperature_client.map(|client| {
            client.callback(Ok(((27.0
                - (((sample as f32 * 3.3 / 65535.0) - self.v_27) * 1000.0 / self.slope))
                * 100.0) as i32));
        });
    }
}

impl<'a, A: adc::AdcChannel<'a>> sensors::TemperatureDriver<'a> for TemperatureRp2040<'a, A> {
    fn set_client(&self, temperature_client: &'a dyn sensors::TemperatureClient) {
        self.temperature_client.replace(temperature_client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.status.set(Status::Read);
            let _ = self.adc.sample();
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}
