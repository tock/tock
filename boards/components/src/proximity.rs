//! Component for any proximity sensor.
//!
//! Usage
//! -----
//! ```rust
//! let proximity = ProximityComponent::new(apds9960, board_kernel, extra_capsules::proximity::DRIVER_NUM)
//!     .finalize(components::proximity_component_static!());
//! ```

use core::mem::MaybeUninit;
use extra_capsules::proximity::ProximitySensor;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

#[macro_export]
macro_rules! proximity_component_static {
    () => {{
        kernel::static_buf!(extra_capsules::proximity::ProximitySensor<'static>)
    };};
}

pub struct ProximityComponent<P: hil::sensors::ProximityDriver<'static> + 'static> {
    sensor: &'static P,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<P: hil::sensors::ProximityDriver<'static>> ProximityComponent<P> {
    pub fn new(
        sensor: &'static P,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> ProximityComponent<P> {
        ProximityComponent {
            board_kernel,
            driver_num,
            sensor,
        }
    }
}

impl<P: hil::sensors::ProximityDriver<'static>> Component for ProximityComponent<P> {
    type StaticInput = &'static mut MaybeUninit<ProximitySensor<'static>>;
    type Output = &'static ProximitySensor<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let proximity = s.write(ProximitySensor::new(self.sensor, grant));

        hil::sensors::ProximityDriver::set_client(self.sensor, proximity);
        proximity
    }
}
