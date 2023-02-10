//! Component for any humidity sensor.
//!
//! Usage
//! -----
//! ```rust
//! let humidity = HumidityComponent::new(board_kernel, nrf52::humidity::TEMP)
//!     .finalize(components::humidity_component_static!());
//! ```

use core::mem::MaybeUninit;
use extra_capsules::humidity::HumiditySensor;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

#[macro_export]
macro_rules! humidity_component_static {
    () => {{
        kernel::static_buf!(extra_capsules::humidity::HumiditySensor<'static>)
    };};
}

pub struct HumidityComponent<T: 'static + hil::sensors::HumidityDriver<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    sensor: &'static T,
}

impl<T: 'static + hil::sensors::HumidityDriver<'static>> HumidityComponent<T> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        sensor: &'static T,
    ) -> HumidityComponent<T> {
        HumidityComponent {
            board_kernel,
            driver_num,
            sensor,
        }
    }
}

impl<T: 'static + hil::sensors::HumidityDriver<'static>> Component for HumidityComponent<T> {
    type StaticInput = &'static mut MaybeUninit<HumiditySensor<'static>>;
    type Output = &'static HumiditySensor<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let humidity = s.write(HumiditySensor::new(
            self.sensor,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        hil::sensors::HumidityDriver::set_client(self.sensor, humidity);
        humidity
    }
}
