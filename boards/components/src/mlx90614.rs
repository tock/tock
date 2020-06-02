//! Components for the MLX90614 IR Temperature Sensor.
//!
//! Usage
//! -----
//! ```rust
//!    let mlx90614 = components::mlx90614::Mlx90614I2CComponent::new()
//!       .finalize(components::mlx90614_i2c_component_helper!(mux_i2c));
//!
//!    let temp = static_init!(
//!           capsules::temperature::TemperatureSensor<'static>,
//!           capsules::temperature::TemperatureSensor::new(mlx90614,
//!                                                    grant_temperature));
//!    kernel::hil::sensors::TemperatureDriver::set_client(mlx90614, temp);
//! ```

use capsules::mlx90614::Mlx90614SMBus;
use capsules::virtual_i2c::{MuxI2C, SMBusDevice};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::{static_init, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! mlx90614_component_helper {
    () => {{
        use capsules::mlx90614::Mlx90614SMBus;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<Mlx90614SMBus<'static>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct Mlx90614SMBusComponent {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
}

impl Mlx90614SMBusComponent {
    pub fn new(i2c: &'static MuxI2C<'static>, i2c_address: u8) -> Self {
        Mlx90614SMBusComponent {
            i2c_mux: i2c,
            i2c_address: i2c_address,
        }
    }
}

static mut I2C_BUF: [u8; 14] = [0; 14];

impl Component for Mlx90614SMBusComponent {
    type StaticInput = &'static mut MaybeUninit<Mlx90614SMBus<'static>>;
    type Output = &'static Mlx90614SMBus<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let mlx90614_smbus = static_init!(
            SMBusDevice,
            SMBusDevice::new(self.i2c_mux, self.i2c_address)
        );
        let mlx90614 = static_init_half!(
            static_buffer,
            Mlx90614SMBus<'static>,
            Mlx90614SMBus::new(mlx90614_smbus, &mut I2C_BUF)
        );

        mlx90614_smbus.set_client(mlx90614);
        mlx90614
    }
}
