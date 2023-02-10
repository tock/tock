//! Components for the BME280 Humidity, Pressure and Temperature Sensor.
//!
//! Usage
//! -----
//! ```rust
//!     let bme280 =
//!         Bme280Component::new(mux_i2c, 0x77).finalize(components::bme280_component_static!());
//!     let temperature = components::temperature::TemperatureComponent::new(
//!         board_kernel,
//!         extra_capsules::temperature::DRIVER_NUM,
//!         bme280,
//!     )
//!     .finalize(components::temperature_component_static!());
//!     let humidity = components::humidity::HumidityComponent::new(
//!         board_kernel,
//!         extra_capsules::humidity::DRIVER_NUM,
//!         bme280,
//!     )
//!     .finalize(components::humidity_component_static!());
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_i2c::{I2CDevice, MuxI2C};
use extra_capsules::bme280::Bme280;
use kernel::component::Component;

// Setup static space for the objects.
#[macro_export]
macro_rules! bme280_component_static {
    () => {{
        let i2c_device = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice<'static>);
        let i2c_buffer = kernel::static_buf!([u8; 26]);
        let bme280 = kernel::static_buf!(extra_capsules::bme280::Bme280<'static>);

        (i2c_device, i2c_buffer, bme280)
    };};
}

pub struct Bme280Component {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
}

impl Bme280Component {
    pub fn new(i2c: &'static MuxI2C<'static>, i2c_address: u8) -> Self {
        Bme280Component {
            i2c_mux: i2c,
            i2c_address: i2c_address,
        }
    }
}

impl Component for Bme280Component {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<[u8; 26]>,
        &'static mut MaybeUninit<Bme280<'static>>,
    );
    type Output = &'static Bme280<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let bme280_i2c = s.0.write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let i2c_buffer = s.1.write([0; 26]);

        let bme280 = s.2.write(Bme280::new(bme280_i2c, i2c_buffer));

        bme280_i2c.set_client(bme280);
        bme280.startup();
        bme280
    }
}
