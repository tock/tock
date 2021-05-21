//! Components for the LSM303DLHC sensor.
//!
//! I2C Interface
//!
//! Usage
//! -----
//! ```rust
//! let lsm303dlhc = components::lsm303dlhc::Lsm303dlhcI2CComponent::new()
//!    .finalize(components::lsm303dlhc_i2c_component_helper!(mux_i2c));
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
use capsules::lsm303dlhc::Lsm303dlhcI2C;
use capsules::virtual_i2c::I2CDevice;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! lsm303dlhc_i2c_component_helper {
    ($i2c_mux:expr $(,)?) => {{
        use capsules::lsm303dlhc::Lsm303dlhcI2C;
        use capsules::virtual_i2c::I2CDevice;
        use core::mem::MaybeUninit;

        static mut BUFFER: [u8; 8] = [0; 8];

        let accelerometer_i2c = components::i2c::I2CComponent::new(
            $i2c_mux,
            capsules::lsm303xx::ACCELEROMETER_BASE_ADDRESS,
        )
        .finalize(components::i2c_component_helper!());
        let magnetometer_i2c = components::i2c::I2CComponent::new(
            $i2c_mux,
            capsules::lsm303xx::MAGNETOMETER_BASE_ADDRESS,
        )
        .finalize(components::i2c_component_helper!());
        static mut lsm303dlhc: MaybeUninit<Lsm303dlhcI2C<'static>> = MaybeUninit::uninit();
        (
            &accelerometer_i2c,
            &magnetometer_i2c,
            &mut BUFFER,
            &mut lsm303dlhc,
        )
    };};
}

pub struct Lsm303dlhcI2CComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl Lsm303dlhcI2CComponent {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize) -> Lsm303dlhcI2CComponent {
        Lsm303dlhcI2CComponent {
            board_kernel,
            driver_num,
        }
    }
}

impl Component for Lsm303dlhcI2CComponent {
    type StaticInput = (
        &'static I2CDevice<'static>,
        &'static I2CDevice<'static>,
        &'static mut [u8],
        &'static mut MaybeUninit<Lsm303dlhcI2C<'static>>,
    );
    type Output = &'static Lsm303dlhcI2C<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap =
            kernel::create_capability!(kernel::capabilities::MemoryAllocationCapability);
        let lsm303dlhc = static_init_half!(
            static_buffer.3,
            Lsm303dlhcI2C<'static>,
            Lsm303dlhcI2C::new(
                static_buffer.0,
                static_buffer.1,
                static_buffer.2,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
            )
        );
        static_buffer.0.set_client(lsm303dlhc);
        static_buffer.1.set_client(lsm303dlhc);

        lsm303dlhc
    }
}
