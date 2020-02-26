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

pub struct DebugProcessRestart<'ker, C: ProcessManagementCapability> {
    kernel: &'ker Kernel<'ker>,
    capability: C,
}

impl<'a, 'ker, C: ProcessManagementCapability> DebugProcessRestart<'ker, C> {
    pub fn new(
        kernel: &'ker Kernel<'ker>,
        pin: &'a dyn gpio::InterruptPin,
        cap: C,
    ) -> DebugProcessRestart<'ker, C> {
        pin.make_input();
        pin.enable_interrupts(gpio::InterruptEdge::RisingEdge);

        DebugProcessRestart {
            kernel: kernel,
            capability: cap,
        }
    }
}

impl<'a, 'ker, C: ProcessManagementCapability> gpio::Client for DebugProcessRestart<'ker, C> {
    fn fired(&self) {
        self.kernel.hardfault_all_apps(&self.capability);
    }
}
