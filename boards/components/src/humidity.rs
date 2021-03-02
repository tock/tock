//! Component for any Temperature sensor.
//!
//! Usage
//! -----
//! ```rust
//! let humidity = HumidityComponent::new(board_kernel, nrf52::humidity::TEMP).finalize(());
//! ```

use capsules::humidity::HumiditySensor;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::static_init;

pub struct HumidityComponent<T: 'static + hil::sensors::HumidityDriver<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: u32,
    temp_sensor: &'static T,
}

impl<T: 'static + hil::sensors::HumidityDriver<'static>> HumidityComponent<T> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: u32,
        temp_sensor: &'static T,
    ) -> HumidityComponent<T> {
        HumidityComponent {
            board_kernel,
            driver_num,
            temp_sensor,
        }
    }
}

impl<T: 'static + hil::sensors::HumidityDriver<'static>> Component for HumidityComponent<T> {
    type StaticInput = ();
    type Output = &'static HumiditySensor<'static>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let humidity = static_init!(
            HumiditySensor<'static>,
            HumiditySensor::new(
                self.temp_sensor,
                self.board_kernel.create_grant(self.driver_num, &grant_cap)
            )
        );

        hil::sensors::HumidityDriver::set_client(self.temp_sensor, humidity);
        humidity
    }
}
