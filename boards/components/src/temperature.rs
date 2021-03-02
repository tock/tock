//! Component for any Temperature sensor.
//!
//! Usage
//! -----
//! ```rust
//! let temp = TemperatureComponent::new(board_kernel, nrf52::temperature::TEMP).finalize(());
//! ```

use capsules::temperature::TemperatureSensor;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::static_init;

pub struct TemperatureComponent<T: 'static + hil::sensors::TemperatureDriver<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: u32,
    temp_sensor: &'static T,
}

impl<T: 'static + hil::sensors::TemperatureDriver<'static>> TemperatureComponent<T> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: u32,
        temp_sensor: &'static T,
    ) -> TemperatureComponent<T> {
        TemperatureComponent {
            board_kernel,
            driver_num,
            temp_sensor,
        }
    }
}

impl<T: 'static + hil::sensors::TemperatureDriver<'static>> Component for TemperatureComponent<T> {
    type StaticInput = ();
    type Output = &'static TemperatureSensor<'static>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let temp = static_init!(
            TemperatureSensor<'static>,
            TemperatureSensor::new(
                self.temp_sensor,
                self.board_kernel.create_grant(self.driver_num, &grant_cap)
            )
        );

        hil::sensors::TemperatureDriver::set_client(self.temp_sensor, temp);
        temp
    }
}
