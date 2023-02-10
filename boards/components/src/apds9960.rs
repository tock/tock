//! Component for APDS9960 proximity sensor.

use core::mem::MaybeUninit;
use core_capsules::virtual_i2c::{I2CDevice, MuxI2C};
use extra_capsules::apds9960::APDS9960;
use kernel::component::Component;
use kernel::hil::gpio;

#[macro_export]
macro_rules! apds9960_component_static {
    () => {{
        let i2c_device = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice<'static>);
        let apds9960 = kernel::static_buf!(extra_capsules::apds9960::APDS9960<'static>);
        let buffer = kernel::static_buf!([u8; extra_capsules::apds9960::BUF_LEN]);

        (i2c_device, apds9960, buffer)
    };};
}

pub struct Apds9960Component {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
    interrupt_pin: &'static dyn gpio::InterruptPin<'static>,
}

impl Apds9960Component {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static>,
        i2c_address: u8,
        interrupt_pin: &'static dyn gpio::InterruptPin<'static>,
    ) -> Self {
        Apds9960Component {
            i2c_mux,
            i2c_address,
            interrupt_pin,
        }
    }
}

impl Component for Apds9960Component {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<APDS9960<'static>>,
        &'static mut MaybeUninit<[u8; extra_capsules::apds9960::BUF_LEN]>,
    );
    type Output = &'static APDS9960<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let apds9960_i2c = s.0.write(I2CDevice::new(self.i2c_mux, self.i2c_address));

        let buffer = s.2.write([0; extra_capsules::apds9960::BUF_LEN]);

        let apds9960 =
            s.1.write(APDS9960::new(apds9960_i2c, self.interrupt_pin, buffer));
        apds9960_i2c.set_client(apds9960);
        self.interrupt_pin.set_client(apds9960);

        apds9960
    }
}
