//! Components for the Ft6x06 Touch Panel.
//!
//! Usage
//! -----
//! ```rust
//! let ft6x06 = components::ft6x06::Ft6x06Component::new()
//!    .finalize(components::ft6x06_i2c_component_helper!(mux_i2c));

//! ```
use capsules::ft6x06::Ft6x06;
use capsules::virtual_i2c::I2CDevice;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::gpio;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! ft6x06_i2c_component_helper {
    ($i2c_mux: expr) => {{
        use capsules::ft6x06::Ft6x06;
        use capsules::virtual_i2c::I2CDevice;
        use core::mem::MaybeUninit;
        let i2c = components::i2c::I2CComponent::new($i2c_mux, 0x38)
            .finalize(components::i2c_component_helper!());
        static mut ft6x06: MaybeUninit<Ft6x06<'static>> = MaybeUninit::uninit();
        (&i2c, &mut ft6x06)
    };};
}

pub struct Ft6x06Component {
    interupt_pin: &'static dyn gpio::InterruptPin,
}

impl Ft6x06Component {
    pub fn new(pin: &'static dyn gpio::InterruptPin) -> Ft6x06Component {
        Ft6x06Component { interupt_pin: pin }
    }
}

impl Component for Ft6x06Component {
    type StaticInput = (
        &'static I2CDevice<'static>,
        &'static mut MaybeUninit<Ft6x06<'static>>,
    );
    type Output = &'static Ft6x06<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let ft6x06 = static_init_half!(
            static_buffer.1,
            Ft6x06<'static>,
            Ft6x06::new(
                static_buffer.0,
                self.interupt_pin,
                &mut capsules::ft6x06::BUFFER
            )
        );
        static_buffer.0.set_client(ft6x06);
        self.interupt_pin.set_client(ft6x06);

        ft6x06
    }
}
