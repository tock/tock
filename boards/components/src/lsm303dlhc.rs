//! Components for the LSM303DLHC sensor.
//!
//! I2C Interface
//!
//! Usage
//! -----
//! ```rust
//! let lsm303dlhc = components::lsm303dlhc::Lsm303dlhcI2CComponent::new().finalize(
//!     components::lsm303dlhc_i2c_component_helper!(i2c_mux)
//! );
//! ```
use capsules::lsm303dlhc::Lsm303dlhc;
use capsules::virtual_i2c::{I2CDevice};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! lsm303dlhc_i2c_component_helper {
    ($i2c_mux: expr) => {{
        use capsules::lsm303dlhc::Lsm303dlhc;
        use capsules::virtual_i2c::I2CDevice;
        use core::mem::MaybeUninit;
        let accelerometer_i2c = components::i2c::I2CComponent::new($i2c_mux, 0x19)
            .finalize(components::i2c_component_helper!());
        let magnetometer_i2c = components::i2c::I2CComponent::new($i2c_mux, 0x1e)
            .finalize(components::i2c_component_helper!());
        static mut lsm303dlhc: MaybeUninit<Lsm303dlhc<'static>> = MaybeUninit::uninit();
        (
            &accelerometer_i2c,
            &magnetometer_i2c,
            &mut lsm303dlhc,
        )
    };};
}

pub struct Lsm303dlhcI2CComponent {}

impl Lsm303dlhcI2CComponent {
    pub fn new() -> Lsm303dlhcI2CComponent {
        Lsm303dlhcI2CComponent {}
    }
}

impl Component for Lsm303dlhcI2CComponent {
    type StaticInput = (
        &'static I2CDevice<'static>,
        &'static I2CDevice<'static>,
        &'static mut MaybeUninit<Lsm303dlhc<'static>>,
    );
    type Output = &'static Lsm303dlhc<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let lsm303dlhc = static_init_half!(
            static_buffer.2,
            Lsm303dlhc<'static>,
            Lsm303dlhc::new(
                static_buffer.0,
                static_buffer.1,
                &mut capsules::lsm303dlhc::BUFFER
            )
        );
        static_buffer.0.set_client(lsm303dlhc);

        lsm303dlhc
    }
}
