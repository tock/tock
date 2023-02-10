//! Component for LPS25HB pressure sensor.

use core::mem::MaybeUninit;
use core_capsules::virtual_i2c::{I2CDevice, MuxI2C};
use extra_capsules::lps25hb::LPS25HB;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::gpio;

#[macro_export]
macro_rules! lps25hb_component_static {
    () => {{
        let i2c_device = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice<'static>);
        let lps25hb = kernel::static_buf!(extra_capsules::lps25hb::LPS25HB<'static>);
        let buffer = kernel::static_buf!([u8; extra_capsules::lps25hb::BUF_LEN]);

        (i2c_device, lps25hb, buffer)
    };};
}

pub struct Lps25hbComponent {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
    interrupt_pin: &'static dyn gpio::InterruptPin<'static>,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl Lps25hbComponent {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static>,
        i2c_address: u8,
        interrupt_pin: &'static dyn gpio::InterruptPin<'static>,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Self {
        Lps25hbComponent {
            i2c_mux,
            i2c_address,
            interrupt_pin,
            board_kernel,
            driver_num,
        }
    }
}

impl Component for Lps25hbComponent {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<LPS25HB<'static>>,
        &'static mut MaybeUninit<[u8; extra_capsules::lps25hb::BUF_LEN]>,
    );
    type Output = &'static LPS25HB<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let lps25hb_i2c = s.0.write(I2CDevice::new(self.i2c_mux, self.i2c_address));

        let buffer = s.2.write([0; extra_capsules::lps25hb::BUF_LEN]);

        let lps25hb =
            s.1.write(LPS25HB::new(lps25hb_i2c, self.interrupt_pin, buffer, grant));
        lps25hb_i2c.set_client(lps25hb);
        self.interrupt_pin.set_client(lps25hb);

        lps25hb
    }
}
