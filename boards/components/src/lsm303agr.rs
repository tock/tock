//! Components for the LSM303DLHC sensor.
//!
//! I2C Interface
//!
//! Usage
//! -----
//! ```rust
//! let lsm303agr = components::lsm303agr::Lsm303agrI2CComponent::new()
//!    .finalize(components::lsm303agr_i2c_component_helper!(mux_i2c));
//!
//! lsm303agr.configure(
//!    lsm303agr::Lsm303dlhcAccelDataRate::DataRate25Hz,
//!    false,
//!    lsm303agr::Lsm303dlhcScale::Scale2G,
//!    false,
//!    true,
//!    lsm303agr::Lsm303dlhcMagnetoDataRate::DataRate3_0Hz,
//!    lsm303agr::Lsm303dlhcRange::Range4_7G,
//! );
//! ```
use capsules::lsm303agr::Lsm303agrI2C;
use capsules::virtual_i2c::I2CDevice;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::{create_capability, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! lsm303agr_i2c_component_helper {
    ($i2c_mux:expr, $accelerometer_address:expr, $magnetometer_address:expr  $(,)?) => {{
        use capsules::lsm303agr::Lsm303agrI2C;
        use capsules::virtual_i2c::I2CDevice;
        use core::mem::MaybeUninit;

        static mut BUFFER: [u8; 8] = [0; 8];

        let accelerometer_i2c =
            components::i2c::I2CComponent::new($i2c_mux, $accelerometer_address)
                .finalize(components::i2c_component_helper!());
        let magnetometer_i2c = components::i2c::I2CComponent::new($i2c_mux, $magnetometer_address)
            .finalize(components::i2c_component_helper!());
        static mut lsm303agr: MaybeUninit<Lsm303agrI2C<'static>> = MaybeUninit::uninit();
        (
            &accelerometer_i2c,
            &magnetometer_i2c,
            &mut BUFFER,
            &mut lsm303agr,
        )
    }};

    ($i2c_mux:expr $(,)?) => {{
        $crate::lsm303agr_i2c_component_helper!(
            $i2c_mux,
            capsules::lsm303xx::ACCELEROMETER_BASE_ADDRESS,
            capsules::lsm303xx::MAGNETOMETER_BASE_ADDRESS
        )
    }};
}

pub struct Lsm303agrI2CComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl Lsm303agrI2CComponent {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize) -> Lsm303agrI2CComponent {
        Lsm303agrI2CComponent {
            board_kernel: board_kernel,
            driver_num,
        }
    }
}

impl Component for Lsm303agrI2CComponent {
    type StaticInput = (
        &'static I2CDevice<'static>,
        &'static I2CDevice<'static>,
        &'static mut [u8],
        &'static mut MaybeUninit<Lsm303agrI2C<'static>>,
    );
    type Output = &'static Lsm303agrI2C<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);
        let lsm303agr = static_init_half!(
            static_buffer.3,
            Lsm303agrI2C<'static>,
            Lsm303agrI2C::new(static_buffer.0, static_buffer.1, static_buffer.2, grant)
        );
        static_buffer.0.set_client(lsm303agr);
        static_buffer.1.set_client(lsm303agr);

        lsm303agr
    }
}
