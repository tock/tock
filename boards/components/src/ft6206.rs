//! Components for the Ft6206 Touch Panel.
//!
//! Usage
//! -----
//! ```rust
//! let ft6206 = components::ft6206::Ft6206Component::new()
//!    .finalize(components::ft6206_i2c_component_helper!(mux_i2c));

//! ```
use capsules::ft6206::Ft6206;
use capsules::virtual_i2c::I2CDevice;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::gpio;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! ft6206_i2c_component_helper {
    ($i2c_mux: expr) => {{
        use capsules::ft6206::Ft6206;
        use capsules::virtual_i2c::I2CDevice;
        use core::mem::MaybeUninit;
        let i2c = components::i2c::I2CComponent::new($i2c_mux, 0x38)
            .finalize(components::i2c_component_helper!());
        static mut ft6206: MaybeUninit<Ft6206<'static>> = MaybeUninit::uninit();
        (&i2c, &mut ft6206)
    };};
}

pub struct Ft6206Component {
    interupt_pin: &'static dyn gpio::InterruptPin,
}

impl Ft6206Component {
    pub fn new(pin: &'static dyn gpio::InterruptPin) -> Ft6206Component {
        Ft6206Component { interupt_pin: pin }
    }
}

impl Component for Ft6206Component {
    type StaticInput = (
        &'static I2CDevice<'static>,
        &'static mut MaybeUninit<Ft6206<'static>>,
    );
    type Output = &'static Ft6206<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let ft6206 = static_init_half!(
            static_buffer.1,
            Ft6206<'static>,
            Ft6206::new(
                static_buffer.0,
                self.interupt_pin,
                &mut capsules::ft6206::BUFFER
            )
        );
        static_buffer.0.set_client(ft6206);
        self.interupt_pin.set_client(ft6206);

        ft6206
    }
}
