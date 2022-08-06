//! Component for any air quality sensor.
//!
//! Usage
//! -----
//! ```rust
//! let temp = AirQualityComponent::new(board_kernel, nrf52::temperature::TEMP).finalize(());
//! ```

use capsules::air_quality::AirQualitySensor;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::static_init;

pub struct AirQualityComponent<T: 'static + hil::sensors::AirQualityDriver<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    temp_sensor: &'static T,
}

impl<T: 'static + hil::sensors::AirQualityDriver<'static>> AirQualityComponent<T> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        temp_sensor: &'static T,
    ) -> AirQualityComponent<T> {
        AirQualityComponent {
            board_kernel,
            driver_num,
            temp_sensor,
        }
    }
}

impl<T: 'static + hil::sensors::AirQualityDriver<'static>> Component for AirQualityComponent<T> {
    type StaticInput = ();
    type Output = &'static AirQualitySensor<'static>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let temp = static_init!(
            AirQualitySensor<'static>,
            AirQualitySensor::new(
                self.temp_sensor,
                self.board_kernel.create_grant(self.driver_num, &grant_cap)
            )
        );

        hil::sensors::AirQualityDriver::set_client(self.temp_sensor, temp);
        temp
    }
}
