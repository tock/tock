//! Debug capsule to cause a button press to make all apps fault.
//!
//! This is useful for debugging that capsules and apps work when they are
//! restarted by the kernel.
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//! # use kernel::capabilities::{Capability, ProcessManagement}
//!
//! let debug_process_restart = static_init!(
//!     capsules::debug_process_restart::DebugProcessRestart<'static>,
//!     capsules::debug_process_restart::DebugProcessRestart::new(
//!         board_kernel,
//!         &sam4l::gpio::PA[16],
//!         kernel::hil::gpio::ActivationMode::ActiveLow,
//!         kernel::hil::gpio::FloatingState::PullUp
//!     )
//! );
//! sam4l::gpio::PA[16].set_client(debug_process_restart);
//! ```

use kernel::capabilities::{Capability, ProcessManagement};
use kernel::hil::gpio;
use kernel::Kernel;

pub struct DebugProcessRestart<'a> {
    kernel: &'static Kernel,
    capability: Capability<ProcessManagement>,
    pin: &'a dyn gpio::InterruptPin<'a>,
    mode: gpio::ActivationMode,
}

impl<'a> DebugProcessRestart<'a> {
    pub fn new(
        kernel: &'static Kernel,
        cap: Capability<ProcessManagement>,
        pin: &'a dyn gpio::InterruptPin<'a>,
        mode: gpio::ActivationMode,
        floating_state: gpio::FloatingState,
    ) -> Self {
        pin.make_input();
        pin.set_floating_state(floating_state);
        pin.enable_interrupts(gpio::InterruptEdge::EitherEdge);

        DebugProcessRestart {
            kernel: kernel,
            capability: cap,
            pin,
            mode,
        }
    }
}

impl gpio::Client for DebugProcessRestart<'_> {
    fn fired(&self) {
        if self.pin.read_activation(self.mode) == gpio::ActivationState::Active {
            self.kernel.hardfault_all_apps(&self.capability);
        }
    }
}
