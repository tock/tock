// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! SyscallDriver for STM ADC MCU temperature sensor

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

pub struct TemperatureSTM<'a, A: adc::AdcChannel<'a>> {
    adc: &'a A,
    slope: f32,
    v_25: f32,
    temperature_client: OptionalCell<&'a dyn sensors::TemperatureClient>,
    status: Cell<Status>,
}

impl<'a, A: adc::AdcChannel<'a>> TemperatureSTM<'a, A> {
    /// slope - device specific slope found in datasheet
    /// v_25 - voltage at 25 degrees Celsius found in datasheet
    pub fn new(adc: &'a A, slope: f32, v_25: f32) -> TemperatureSTM<'a, A> {
        TemperatureSTM {
            adc: adc,
            slope: slope,
            v_25: v_25,
            temperature_client: OptionalCell::empty(),
            status: Cell::new(Status::Idle),
        }
    }
}

impl<'a, A: adc::AdcChannel<'a>> adc::Client for TemperatureSTM<'a, A> {
    fn sample_ready(&self, sample: u16) {
        self.status.set(Status::Idle);
        self.temperature_client.map(|client| {
            client.callback(Ok(
                ((((self.v_25 - (sample as f32 * 3.3 / 65535.0)) * 1000.0 / self.slope) + 25.0)
                    * 100.0) as i32,
            ));
        });
    }
}

impl<'a, A: adc::AdcChannel<'a>> sensors::TemperatureDriver<'a> for TemperatureSTM<'a, A> {
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
