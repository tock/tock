//! Component for the LSM6DSOXTR Sensor
//!
//! Usage
//! ------
//!
//! ```rust
//! let lsm6dsoxtr = components::lsm6dsox::Lsm6dsoxtrI2CComponent::new(
//!     mux_i2c,
//!     extra_capsules::lsm6dsoxtr::ACCELEROMETER_BASE_ADDRESS,
//!     board_kernel,
//!     extra_capsules::lsm6dsoxtr::DRIVER_NUM,
//! )
//! .finalize(components::lsm6ds_i2c_component_static!());
//!
//! let _ = lsm6dsoxtr
//!          .configure(
//!              extra_capsules::lsm6ds_definitions::LSM6DSOXGyroDataRate::LSM6DSOX_GYRO_RATE_12_5_HZ,
//!              extra_capsules::lsm6ds_definitions::LSM6DSOXAccelDataRate::LSM6DSOX_ACCEL_RATE_12_5_HZ,
//!              extra_capsules::lsm6ds_definitions::LSM6DSOXAccelRange::LSM6DSOX_ACCEL_RANGE_2_G,
//!              extra_capsules::lsm6ds_definitions::LSM6DSOXTRGyroRange::LSM6DSOX_GYRO_RANGE_250_DPS,
//!              true,
//!          )
//!          .map_err(|e| panic!("ERROR Failed LSM6DSOXTR sensor configuration ({:?})", e));
//! ```
//! Author: Cristiana Andrei <cristiana.andrei@stud.fils.upb.ro>

use core::mem::MaybeUninit;
use core_capsules::virtual_i2c::{I2CDevice, MuxI2C};
use extra_capsules::lsm6dsoxtr::Lsm6dsoxtrI2C;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;

// Setup static space for the objects.
#[macro_export]
macro_rules! lsm6ds_i2c_component_static {
    () => {{
        let buffer = kernel::static_buf!([u8; 8]);
        let i2c_device = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice<'static>);
        let lsm6dsoxtr = kernel::static_buf!(extra_capsules::lsm6dsoxtr::Lsm6dsoxtrI2C<'static>);

        (i2c_device, buffer, lsm6dsoxtr)
    };};
}

pub struct Lsm6dsoxtrI2CComponent {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl Lsm6dsoxtrI2CComponent {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static>,
        i2c_address: u8,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Lsm6dsoxtrI2CComponent {
        Lsm6dsoxtrI2CComponent {
            i2c_mux,
            i2c_address,
            board_kernel,
            driver_num,
        }
    }
}

impl Component for Lsm6dsoxtrI2CComponent {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<[u8; 8]>,
        &'static mut MaybeUninit<Lsm6dsoxtrI2C<'static>>,
    );
    type Output = &'static Lsm6dsoxtrI2C<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let lsm6dsox_i2c = static_buffer
            .0
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let buffer = static_buffer.1.write([0; 8]);

        let lsm6dsox = static_buffer
            .2
            .write(Lsm6dsoxtrI2C::new(lsm6dsox_i2c, buffer, grant));
        lsm6dsox_i2c.set_client(lsm6dsox);

        lsm6dsox
    }
}
