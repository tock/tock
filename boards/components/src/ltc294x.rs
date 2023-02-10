//! Component for LPS25HB pressure sensor.
//!
//! Usage
//! -----
//!
//! ```rust
//! let ltc294x = components::Ltc294xComponent::new(i2c_mux, 0x64, None)
//!     .finalize(components::ltc294x_component_static!());
//! let ltc294x_driver = components::Ltc294xDriverComponent::new(ltc294x, board_kernel, DRIVER_NUM)
//!     .finalize(components::ltc294x_driver_component_static!());
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_i2c::{I2CDevice, MuxI2C};
use extra_capsules::ltc294x::LTC294XDriver;
use extra_capsules::ltc294x::LTC294X;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::gpio;

#[macro_export]
macro_rules! ltc294x_component_static {
    () => {{
        let i2c_device = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice<'static>);
        let ltc294x = kernel::static_buf!(extra_capsules::ltc294x::LTC294X<'static>);
        let buffer = kernel::static_buf!([u8; extra_capsules::ltc294x::BUF_LEN]);

        (i2c_device, ltc294x, buffer)
    };};
}

#[macro_export]
macro_rules! ltc294x_driver_component_static {
    () => {{
        kernel::static_buf!(extra_capsules::ltc294x::LTC294XDriver<'static>)
    };};
}

pub struct Ltc294xComponent {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
    interrupt_pin: Option<&'static dyn gpio::InterruptPin<'static>>,
}

impl Ltc294xComponent {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static>,
        i2c_address: u8,
        interrupt_pin: Option<&'static dyn gpio::InterruptPin<'static>>,
    ) -> Self {
        Ltc294xComponent {
            i2c_mux,
            i2c_address,
            interrupt_pin,
        }
    }
}

impl Component for Ltc294xComponent {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<LTC294X<'static>>,
        &'static mut MaybeUninit<[u8; extra_capsules::ltc294x::BUF_LEN]>,
    );
    type Output = &'static LTC294X<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let ltc294x_i2c = s.0.write(I2CDevice::new(self.i2c_mux, self.i2c_address));

        let buffer = s.2.write([0; extra_capsules::ltc294x::BUF_LEN]);

        let ltc294x =
            s.1.write(LTC294X::new(ltc294x_i2c, self.interrupt_pin, buffer));
        ltc294x_i2c.set_client(ltc294x);
        self.interrupt_pin.map(|pin| {
            pin.set_client(ltc294x);
        });

        ltc294x
    }
}

pub struct Ltc294xDriverComponent {
    ltc294x: &'static LTC294X<'static>,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl Ltc294xDriverComponent {
    pub fn new(
        ltc294x: &'static LTC294X<'static>,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Self {
        Ltc294xDriverComponent {
            ltc294x,
            board_kernel,
            driver_num,
        }
    }
}

impl Component for Ltc294xDriverComponent {
    type StaticInput = &'static mut MaybeUninit<LTC294XDriver<'static>>;
    type Output = &'static LTC294XDriver<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let ltc294x_driver = s.write(LTC294XDriver::new(self.ltc294x, grant));
        self.ltc294x.set_client(ltc294x_driver);

        ltc294x_driver
    }
}
