//! Components for the Ft6x06 Touch Panel.
//!
//! Usage
//! -----
//! ```rust
//! let ft6x06 = components::ft6x06::Ft6x06Component::new(
//!    i2c_mux,
//!    0x38,
//!    base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PG05).unwrap()
//! )
//!    .finalize(components::ft6x06_component_static!(mux_i2c));
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_i2c::{I2CDevice, MuxI2C};
use extra_capsules::ft6x06::Ft6x06;
use extra_capsules::ft6x06::NO_TOUCH;
use kernel::component::Component;
use kernel::hil::gpio;

// Setup static space for the objects.
#[macro_export]
macro_rules! ft6x06_component_static {
    () => {{
        let i2c_device = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice);
        let buffer = kernel::static_buf!([u8; 17]);
        let events_buffer = kernel::static_buf!([kernel::hil::touch::TouchEvent; 2]);
        let ft6x06 = kernel::static_buf!(extra_capsules::ft6x06::Ft6x06<'static>);

        (i2c_device, ft6x06, buffer, events_buffer)
    };};
}

pub struct Ft6x06Component {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
    interrupt_pin: &'static dyn gpio::InterruptPin<'static>,
}

impl Ft6x06Component {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static>,
        i2c_address: u8,
        pin: &'static dyn gpio::InterruptPin,
    ) -> Ft6x06Component {
        Ft6x06Component {
            i2c_mux,
            i2c_address,
            interrupt_pin: pin,
        }
    }
}

impl Component for Ft6x06Component {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<Ft6x06<'static>>,
        &'static mut MaybeUninit<[u8; 17]>,
        &'static mut MaybeUninit<[kernel::hil::touch::TouchEvent; 2]>,
    );
    type Output = &'static Ft6x06<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let ft6x06_i2c = static_buffer
            .0
            .write(I2CDevice::new(self.i2c_mux, self.i2c_address));

        let buffer = static_buffer.2.write([0; 17]);
        let events_buffer = static_buffer.3.write([NO_TOUCH, NO_TOUCH]);

        let ft6x06 = static_buffer.1.write(Ft6x06::new(
            ft6x06_i2c,
            self.interrupt_pin,
            buffer,
            events_buffer,
        ));
        ft6x06_i2c.set_client(ft6x06);
        self.interrupt_pin.set_client(ft6x06);

        ft6x06
    }
}
