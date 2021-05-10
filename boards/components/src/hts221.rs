//! Components for the HTS221 Temperature/Humidity Sensor.
//!
//! Usage
//! -----
//! ```rust
//! let hts221 = Hts221Component::new(mux_i2c, mux_alarm, 0x5f).finalize(
//!     components::hts221!(sam4l::ast::Ast));
//! let temperature = components::temperature::TemperatureComponent::new(board_kernel, hts221).finalize(());
//! let humidity = components::humidity::HumidityComponent::new(board_kernel, hts221).finalize(());
//! ```

use core::mem::MaybeUninit;

use capsules::hts221::Hts221;
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use kernel::component::Component;
use kernel::{static_init, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! hts221_component_helper {
    () => {{
        use capsules::hts221::Hts221;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<Hts221<'static>> = MaybeUninit::uninit();
        &mut BUF1
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

static mut I2C_BUF: [u8; 17] = [0; 17];

impl Component for Hts221Component {
    type StaticInput = &'static mut MaybeUninit<Hts221<'static>>;
    type Output = &'static Hts221<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let hts221_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, self.i2c_address));
        let hts221 = static_init_half!(
            static_buffer,
            Hts221<'static>,
            Hts221::new(hts221_i2c, &mut I2C_BUF)
        );

        hts221_i2c.set_client(hts221);
        hts221
    }
}
