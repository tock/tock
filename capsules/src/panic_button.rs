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

pub struct PanicButton<'a> {
    pin: &'a dyn gpio::InterruptPin,
    mode: gpio::ActivationMode,
}

impl<'a> PanicButton<'a> {
    pub fn new(
        pin: &'a dyn gpio::InterruptPin,
        mode: gpio::ActivationMode,
        floating_state: gpio::FloatingState,
    ) -> Self {
        pin.make_input();
        pin.set_floating_state(floating_state);
        pin.enable_interrupts(gpio::InterruptEdge::EitherEdge);

        PanicButton { pin, mode }
    }
}

impl gpio::Client for PanicButton<'_> {
    fn fired(&self) {
        if self.pin.read_activation(self.mode) == gpio::ActivationState::Active {
            panic!("Panic button pressed");
        }
    }
}
