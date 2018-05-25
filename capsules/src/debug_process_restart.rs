//! Debug capsule to cause a button press to restart all apps.
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
use kernel::hil;
use kernel::hil::gpio::{Client, InterruptMode};
use kernel::Kernel;

pub struct DebugProcessRestart<'a, G: hil::gpio::Pin + 'a, C: ProcessManagementCapability> {
    kernel: &'static Kernel,
    _pin: &'a G,
    capability: C,
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl, C: ProcessManagementCapability>
    DebugProcessRestart<'a, G, C>
{
    pub fn new(kernel: &'static Kernel, pin: &'a G, cap: C) -> DebugProcessRestart<'a, G, C> {
        pin.make_input();
        pin.enable_interrupt(0, InterruptMode::RisingEdge);

        DebugProcessRestart {
            kernel: kernel,
            _pin: pin,
            capability: cap,
        }
    }
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl, C: ProcessManagementCapability> Client
    for DebugProcessRestart<'a, G, C>
{
    fn fired(&self, _pin_num: usize) {
        self.kernel.hardfault_all_apps(&self.capability);
    }
}
