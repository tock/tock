//! Components for the BME280 Humidity, Pressure and Temperature Sensor.
//!
//! Usage
//! -----
//! ```rust
//!     let bme280 =
//!         Bme280Component::new(mux_i2c, 0x77).finalize(components::bme280_component_helper!());
//!     let temperature = components::temperature::TemperatureComponent::new(
//!         board_kernel,
//!         capsules::temperature::DRIVER_NUM,
//!         bme280,
//!     )
//!     .finalize(());
//!     let humidity = components::humidity::HumidityComponent::new(
//!         board_kernel,
//!         capsules::humidity::DRIVER_NUM,
//!         bme280,
//!     )
//!     .finalize(());
//! ```

use capsules::bme280::Bme280;
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::{static_init, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! bme280_component_helper {
    () => {{
        use capsules::bme280::Bme280;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<Bme280<'static>> = MaybeUninit::uninit();
        &mut BUF1
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

static mut I2C_BUF: [u8; 26] = [0; 26];

impl Component for Bme280Component {
    type StaticInput = &'static mut MaybeUninit<Bme280<'static>>;
    type Output = &'static Bme280<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let bme280_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, self.i2c_address));
        let bme280 = static_init_half!(
            static_buffer,
            Bme280<'static>,
            Bme280::new(bme280_i2c, &mut I2C_BUF)
        );

        bme280_i2c.set_client(bme280);
        bme280.startup();
        bme280
    }
}
