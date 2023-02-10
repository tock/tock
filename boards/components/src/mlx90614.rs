//! Components for the MLX90614 IR Temperature Sensor.
//!
//! Usage
//! -----
//! ```rust
//! let mlx90614 = components::mlx90614::Mlx90614I2CComponent::new(mux_i2c, i2c_addr,
//! board_kernel)
//!    .finalize(components::mlx90614_component_static!());
//!
//! let temp = static_init!(
//!        extra_capsules::temperature::TemperatureSensor<'static>,
//!        extra_capsules::temperature::TemperatureSensor::new(mlx90614,
//!                                                 grant_temperature));
//! kernel::hil::sensors::TemperatureDriver::set_client(mlx90614, temp);
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_i2c::{MuxI2C, SMBusDevice};
use extra_capsules::mlx90614::Mlx90614SMBus;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;

// Setup static space for the objects.
#[macro_export]
macro_rules! mlx90614_component_static {
    () => {{
        let i2c_device = kernel::static_buf!(core_capsules::virtual_i2c::SMBusDevice);
        let buffer = kernel::static_buf!([u8; 14]);
        let mlx90614 = kernel::static_buf!(extra_capsules::mlx90614::Mlx90614SMBus<'static>);

        (i2c_device, buffer, mlx90614)
    };};
}

pub struct Mlx90614SMBusComponent {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl Mlx90614SMBusComponent {
    pub fn new(
        i2c: &'static MuxI2C<'static>,
        i2c_address: u8,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Self {
        Mlx90614SMBusComponent {
            i2c_mux: i2c,
            i2c_address: i2c_address,
            board_kernel,
            driver_num,
        }
    }
}

impl Component for Mlx90614SMBusComponent {
    type StaticInput = (
        &'static mut MaybeUninit<SMBusDevice<'static>>,
        &'static mut MaybeUninit<[u8; 14]>,
        &'static mut MaybeUninit<Mlx90614SMBus<'static>>,
    );
    type Output = &'static Mlx90614SMBus<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let mlx90614_smbus = static_buffer
            .0
            .write(SMBusDevice::new(self.i2c_mux, self.i2c_address));
        let buffer = static_buffer.1.write([0; 14]);
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let mlx90614 = static_buffer.2.write(Mlx90614SMBus::new(
            mlx90614_smbus,
            buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        mlx90614_smbus.set_client(mlx90614);
        mlx90614
    }
}
