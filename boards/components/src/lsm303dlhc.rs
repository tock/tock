//! Components for the LSM303DLHC sensor.
//!
//! I2C Interface
//!
//! Usage
//! -----
//! ```rust
//! let lsm303dlhc = components::lsm303dlhc::Lsm303dlhcI2CComponent::new(i2c_mux, board_kernel, driver_num)
//!    .finalize(components::lsm303dlhc_component_static!());
//!
//! lsm303dlhc.configure(
//!    lsm303dlhc::Lsm303dlhcAccelDataRate::DataRate25Hz,
//!    false,
//!    lsm303dlhc::Lsm303dlhcScale::Scale2G,
//!    false,
//!    true,
//!    lsm303dlhc::Lsm303dlhcMagnetoDataRate::DataRate3_0Hz,
//!    lsm303dlhc::Lsm303dlhcRange::Range4_7G,
//! );
//! ```
use core::mem::MaybeUninit;
use core_capsules::virtual_i2c::{I2CDevice, MuxI2C};
use extra_capsules::lsm303dlhc::Lsm303dlhcI2C;
use extra_capsules::lsm303xx;
use kernel::component::Component;

// Setup static space for the objects.
#[macro_export]
macro_rules! lsm303dlhc_component_static {
    () => {{
        let buffer = kernel::static_buf!([u8; 8]);
        let accelerometer_i2c = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice);
        let magnetometer_i2c = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice);
        let lsm303dlhc = kernel::static_buf!(extra_capsules::lsm303dlhc::Lsm303dlhcI2C<'static>);

        (accelerometer_i2c, magnetometer_i2c, buffer, lsm303dlhc)
    };};
}

pub struct Lsm303dlhcI2CComponent {
    i2c_mux: &'static MuxI2C<'static>,
    accelerometer_i2c_address: u8,
    magnetometer_i2c_address: u8,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl Lsm303dlhcI2CComponent {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static>,
        accelerometer_i2c_address: Option<u8>,
        magnetometer_i2c_address: Option<u8>,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Lsm303dlhcI2CComponent {
        Lsm303dlhcI2CComponent {
            i2c_mux,
            accelerometer_i2c_address: accelerometer_i2c_address
                .unwrap_or(lsm303xx::ACCELEROMETER_BASE_ADDRESS),
            magnetometer_i2c_address: magnetometer_i2c_address
                .unwrap_or(lsm303xx::MAGNETOMETER_BASE_ADDRESS),
            board_kernel,
            driver_num,
        }
    }
}

impl Component for Lsm303dlhcI2CComponent {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<[u8; 8]>,
        &'static mut MaybeUninit<Lsm303dlhcI2C<'static>>,
    );
    type Output = &'static Lsm303dlhcI2C<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap =
            kernel::create_capability!(kernel::capabilities::MemoryAllocationCapability);

        let buffer = static_buffer.2.write([0; 8]);

        let accelerometer_i2c = static_buffer
            .0
            .write(I2CDevice::new(self.i2c_mux, self.accelerometer_i2c_address));
        let magnetometer_i2c = static_buffer
            .1
            .write(I2CDevice::new(self.i2c_mux, self.magnetometer_i2c_address));

        let lsm303dlhc = static_buffer.3.write(Lsm303dlhcI2C::new(
            accelerometer_i2c,
            magnetometer_i2c,
            buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        accelerometer_i2c.set_client(lsm303dlhc);
        magnetometer_i2c.set_client(lsm303dlhc);

        lsm303dlhc
    }
}
