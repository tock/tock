//! Capsule for analog sensors.
//!
//! This capsule provides the sensor HIL interfaces for sensors which only need
//! an ADC.
//!
//! It includes support for analog light sensors and analog temperature sensors.

use kernel::common::cells::OptionalCell;
use kernel::hil;
use kernel::ReturnCode;

/// The type of the sensor implies how the raw ADC reading should be converted
/// to a light value.
pub enum AnalogLightSensorType {
    LightDependentResistor,
}

pub struct AnalogLightSensor<'a, A: hil::adc::Adc> {
    adc: &'a A,
    channel: &'a <A as hil::adc::Adc>::Channel,
    sensor_type: AnalogLightSensorType,
    client: OptionalCell<&'a dyn hil::sensors::AmbientLightClient>,
}

impl<'a, A: hil::adc::Adc> AnalogLightSensor<'a, A> {
    pub fn new(
        adc: &'a A,
        channel: &'a <A as hil::adc::Adc>::Channel,
        sensor_type: AnalogLightSensorType,
    ) -> AnalogLightSensor<'a, A> {
        AnalogLightSensor {
            adc: adc,
            channel: channel,
            sensor_type: sensor_type,
            client: OptionalCell::empty(),
        }
    }
}

/// Callbacks from the ADC driver
impl<A: hil::adc::Adc> hil::adc::Client for AnalogLightSensor<'_, A> {
    fn sample_ready(&self, sample: u16) {
        // TODO: calculate the actual light reading.
        let measurement: usize = match self.sensor_type {
            AnalogLightSensorType::LightDependentResistor => {
                // TODO: need to determine the actual value that the 5000 should be
                (sample as usize * 5000) / 65535
            }
        };
        self.client.map(|client| client.callback(measurement));
    }
}

impl<A: hil::adc::Adc> hil::sensors::AmbientLight for AnalogLightSensor<'_, A> {
    fn set_client(&self, client: &'static dyn hil::sensors::AmbientLightClient) {
        self.client.set(client);
    }

    fn read_light_intensity(&self) -> ReturnCode {
        self.adc.sample(self.channel)
    }
}

/// The type of the sensor implies how the raw ADC reading should be converted
/// to a temperature value.
pub enum AnalogTemperatureSensorType {
    MicrochipMcp9700,
}

pub struct AnalogTemperatureSensor<'a, A: hil::adc::Adc> {
    adc: &'a A,
    channel: &'a <A as hil::adc::Adc>::Channel,
    sensor_type: AnalogTemperatureSensorType,
    client: OptionalCell<&'a dyn hil::sensors::TemperatureClient>,
}

impl<'a, A: hil::adc::Adc> AnalogTemperatureSensor<'a, A> {
    pub fn new(
        adc: &'a A,
        channel: &'a <A as hil::adc::Adc>::Channel,
        sensor_type: AnalogLightSensorType,
    ) -> AnalogLightSensor<'a, A> {
        AnalogLightSensor {
            adc: adc,
            channel: channel,
            sensor_type: sensor_type,
            client: OptionalCell::empty(),
        }
    }
}

/// Callbacks from the ADC driver
impl<A: hil::adc::Adc> hil::adc::Client for AnalogTemperatureSensor<'_, A> {
    fn sample_ready(&self, sample: u16) {
        // TODO: calculate the actual temperature reading.
        let measurement: usize = match self.sensor_type {
            // 𝑉out = 500𝑚𝑉 + 10𝑚𝑉/C ∗ 𝑇A
            AnalogTemperatureSensorType::MicrochipMcp9700 => {
                let ref_mv = self.adc.get_voltage_reference_mv().unwrap_or(3300);
                // reading_mv = (ADC / (2^16-1)) * ref_voltage
                let reading_mv = (sample as usize * ref_mv) / 65535;
                // need 0.01°C
                (reading_mv - 500) * 10
            }
        };
        self.client.map(|client| client.callback(measurement));
    }
}

impl<A: hil::adc::Adc> hil::sensors::TemperatureDriver for AnalogTemperatureSensor<'_, A> {
    fn set_client(&self, client: &'static dyn hil::sensors::TemperatureClient) {
        self.client.set(client);
    }

    fn read_temperature(&self) -> ReturnCode {
        self.adc.sample(self.channel)
    }
}
