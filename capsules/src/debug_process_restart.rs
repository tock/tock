//! Debug capsule to cause a button press to make all apps fault.
//!
//! This is useful for debugging that capsules and apps work when they are
//! restarted by the kernel.
//!
//! Usage
//! -----
//!
//! ```rust
//! struct ProcessMgmtCap;
//! unsafe impl capabilities::ProcessManagementCapability for ProcessMgmtCap {}
//! let debug_process_restart = static_init!(
//!     capsules::debug_process_restart::DebugProcessRestart<
//!         'static,
//!         sam4l::gpio::GPIOPin,
//!         ProcessMgmtCap,
//!     >,
//!     capsules::debug_process_restart::DebugProcessRestart::new(
//!         board_kernel,
//!         &sam4l::gpio::PA[16],
//!         ProcessMgmtCap
//!     )
//! );
//! sam4l::gpio::PA[16].set_client(debug_process_restart);
//! ```

use kernel::capabilities::ProcessManagementCapability;
use kernel::hil::gpio;
use kernel::Kernel;

pub struct DebugProcessRestart<C: ProcessManagementCapability> {
    kernel: &'static Kernel,
    capability: C,
}

impl<'a, C: ProcessManagementCapability> DebugProcessRestart<C> {
    pub fn new(kernel: &'static Kernel, pin: &'a gpio::InterruptPin, cap: C) -> DebugProcessRestart<C> {
        pin.make_input();
        pin.enable_interrupt(0, gpio::InterruptEdge::RisingEdge);

        DebugProcessRestart {
            kernel: kernel,
            capability: cap,
        }
    }
}

impl<'a, C: ProcessManagementCapability> gpio::Client for DebugProcessRestart<C> {
    fn fired(&self) {
        self.kernel.hardfault_all_apps(&self.capability);
    }
}
