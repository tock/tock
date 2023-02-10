//! Component to cause a button press to trigger a kernel panic.
//!
//! This can be useful especially when developing or debugging console
//! capsules.
//!
//! Note: the process console has support for triggering a panic, which may be
//! more convenient depending on the board.
//!
//! Usage
//! -----
//!
//! ```rust
//! components::panic_button::PanicButtonComponent::new(
//!     &sam4l::gpio::PC[24],
//!     kernel::hil::gpio::ActivationMode::ActiveLow,
//!     kernel::hil::gpio::FloatingState::PullUp
//! )
//! .finalize(components::panic_button_component_static!(sam4l::gpio::GPIOPin));
//! ```

use core::mem::MaybeUninit;
use extra_capsules::panic_button::PanicButton;
use kernel::component::Component;
use kernel::hil::gpio;

#[macro_export]
macro_rules! panic_button_component_static {
    ($Pin:ty $(,)?) => {{
        kernel::static_buf!(core_capsules::button::PanicButton<'static, $Pin>)
    };};
}

pub struct PanicButtonComponent<'a, IP: gpio::InterruptPin<'a>> {
    pin: &'a IP,
    mode: gpio::ActivationMode,
    floating_state: gpio::FloatingState,
}

impl<'a, IP: gpio::InterruptPin<'a>> PanicButtonComponent<'a, IP> {
    pub fn new(
        pin: &'a IP,
        mode: gpio::ActivationMode,
        floating_state: gpio::FloatingState,
    ) -> Self {
        Self {
            pin,
            mode,
            floating_state,
        }
    }
}

impl<IP: 'static + gpio::InterruptPin<'static>> Component for PanicButtonComponent<'static, IP> {
    type StaticInput = &'static mut MaybeUninit<PanicButton<'static, IP>>;
    type Output = ();

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let panic_button =
            static_buffer.write(PanicButton::new(self.pin, self.mode, self.floating_state));
        self.pin.set_client(panic_button);
    }
}
