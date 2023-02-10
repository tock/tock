//! Components for the HTS221 Temperature/Humidity Sensor.
//!
//! Usage
//! -----
//! ```rust
//! let hts221 = Hts221Component::new(mux_i2c, mux_alarm, 0x5f).finalize(
//!     components::hts221_component_static!(sam4l::ast::Ast));
//! let temperature = components::temperature::TemperatureComponent::new(board_kernel, hts221).finalize(());
//! let humidity = components::humidity::HumidityComponent::new(board_kernel, hts221).finalize(());
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_i2c::{I2CDevice, MuxI2C};
use extra_capsules::hts221::Hts221;
use kernel::component::Component;

// Setup static space for the objects.
#[macro_export]
macro_rules! hts221_component_static {
    () => {{
        let i2c_device = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice);
        let buffer = kernel::static_buf!([u8; 17]);
        let hts221 = kernel::static_buf!(extra_capsules::hts221::Hts221<'static>);

        (i2c_device, buffer, hts221)
    };};
}

pub struct Hts221Component {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
}

impl Hts221Component {
    pub fn new(i2c: &'static MuxI2C<'static>, i2c_address: u8) -> Self {
        Hts221Component {
            i2c_mux: i2c,
            i2c_address: i2c_address,
        }
    }
}

impl Component for Hts221Component {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<[u8; 17]>,
        &'static mut MaybeUninit<Hts221<'static>>,
    );
    type Output = &'static Hts221<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let hts221_i2c = static_buffer
            .0
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let buffer = static_buffer.1.write([0; 17]);
        let hts221 = static_buffer.2.write(Hts221::new(hts221_i2c, buffer));

        hts221_i2c.set_client(hts221);
        hts221
    }
}
