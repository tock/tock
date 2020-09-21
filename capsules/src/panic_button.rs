//! Debug capsule to cause a button press to trigger a kernel panic.
//!
//! This can be useful especially when developping or debugging console
//! capsules.
//!
//! Usage
//! -----
//!
//! The recommended way is to use the `PanicButtonComponent`.
//!
//! Alternatively, a low-level way of using the capsule is as follows.
//!
//! ```rust
//! let panic_button = static_init!(
//!     PanicButton,
//!     PanicButton::new(
//!         &sam4l::gpio::PA[16],
//!         kernel::hil::gpio::ActivationMode::ActiveLow,
//!         kernel::hil::gpio::FloatingState::PullUp
//!     )
//! );
//! sam4l::gpio::PA[16].set_client(panic_button);
//! ```

use kernel::hil::gpio;

pub struct PanicButton<'a, IP: gpio::InterruptPin<'a>> {
    pin: &'a IP,
    mode: gpio::ActivationMode,
}

impl<'a, IP: gpio::InterruptPin<'a>> PanicButton<'a, IP> {
    pub fn new(
        pin: &'a IP,
        mode: gpio::ActivationMode,
        floating_state: gpio::FloatingState,
    ) -> Self {
        pin.make_input();
        pin.set_floating_state(floating_state);
        pin.enable_interrupts(gpio::InterruptEdge::EitherEdge);

        Self { pin, mode }
    }
}

impl<'a, IP: gpio::InterruptPin<'a>> gpio::Client for PanicButton<'a, IP> {
    fn fired(&self) {
        if self.pin.read_activation(self.mode) == gpio::ActivationState::Active {
            panic!("Panic button pressed");
        }
    }
}
