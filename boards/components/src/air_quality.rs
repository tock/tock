//! Component for any air quality sensor.
//!
//! Usage
//! -----
//! ```rust
//! let temp = AirQualityComponent::new(board_kernel, nrf52::temperature::TEMP)
//!     .finalize(air_quality_component_static!());
//! ```

use core::mem::MaybeUninit;
use extra_capsules::air_quality::AirQualitySensor;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

#[macro_export]
macro_rules! air_quality_component_static {
    () => {{
        kernel::static_buf!(extra_capsules::air_quality::AirQualitySensor<'static>)
    };};
}

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
    type StaticInput = &'static mut MaybeUninit<AirQualitySensor<'static>>;
    type Output = &'static AirQualitySensor<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let air_quality = s.write(AirQualitySensor::new(
            self.temp_sensor,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        hil::sensors::AirQualityDriver::set_client(self.temp_sensor, air_quality);
        air_quality
    }
}
