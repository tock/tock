//! Driver for STM ADC MCU temperature sensor

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::hil::adc;
use kernel::hil::sensors;
use kernel::ReturnCode;

use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Temperature as usize;

#[derive(Copy, Clone, PartialEq)]
pub enum Status {
    Read,
    Idle,
}

pub struct TemperatureSTM<'a> {
    adc: &'a dyn adc::AdcChannel,
    slope: f32,
    v_25: f32,
    temperature_client: OptionalCell<&'a dyn sensors::TemperatureClient>,
    status: Cell<Status>,
}

impl<'a> TemperatureSTM<'a> {
    /// slope - device specific slope found in datasheet
    /// v_25 - voltage at 25 degreeds celcius found in datasheet
    pub fn new(adc: &'a dyn adc::AdcChannel, slope: f32, v_25: f32) -> TemperatureSTM<'a> {
        TemperatureSTM {
            adc: adc,
            slope: slope,
            v_25: v_25,
            temperature_client: OptionalCell::empty(),
            status: Cell::new(Status::Idle),
        }
    }
}

impl<'a> adc::Client for TemperatureSTM<'a> {
    fn sample_ready(&self, sample: u16) {
        self.status.set(Status::Idle);
        self.temperature_client.map(|client| {
            client.callback(
                ((((self.v_25 - (sample as f32 * 3.3 / 4095.0)) * 1000.0 / self.slope) + 25.0)
                    * 100.0) as usize,
            );
        });
    }
}

impl<'a> sensors::TemperatureDriver<'a> for TemperatureSTM<'a> {
    fn set_client(&self, temperature_client: &'a dyn sensors::TemperatureClient) {
        self.temperature_client.replace(temperature_client);
    }

    fn read_temperature(&self) -> ReturnCode {
        if self.status.get() == Status::Idle {
            self.status.set(Status::Read);
            self.adc.sample();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }
}
