//! Interfaces for implementing boards in Tock.

use crate::driver::Driver;
use crate::errorcode;
use crate::process;
use crate::syscall;

/// Interface for individual boards.
///
/// Each board should define a struct which implements this trait. This trait is
/// the core for how syscall dispatching is handled, and the implementation is
/// responsible for dispatching to drivers for each system call number.
///
/// ## Example
///
/// ```ignore
/// struct Hail {
///     console: &'static capsules::console::Console<'static>,
///     ipc: kernel::ipc::IPC,
///     dac: &'static capsules::dac::Dac<'static>,
/// }
///
/// impl Platform for Hail {
///     fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
///     where
///         F: FnOnce(Option<&dyn kernel::Driver>) -> R,
///     {
///         match driver_num {
///             capsules::console::DRIVER_NUM => f(Some(self.console)),
///             kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
///             capsules::dac::DRIVER_NUM => f(Some(self.dac)),
///
///             _ => f(None),
///         }
///     }
/// }
/// ```
pub trait Platform {
    /// Platform-specific mapping of syscall numbers to objects that implement
    /// the Driver methods for that syscall.
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn Driver>) -> R;

    /// Check the platform-provided system call filter for all non-yield system
    /// calls. If the system call is allowed for the provided process then
    /// return `Ok(())`. Otherwise, return `Err()` with an `ErrorCode` that will
    /// be returned to the calling application. The default implementation
    /// allows all system calls.
    ///
    /// This API should be considered unstable, and is likely to change in the
    /// future.
    fn filter_syscall(
        &self,
        _process: &dyn process::Process,
        _syscall: &syscall::Syscall,
    ) -> Result<(), errorcode::ErrorCode> {
        Ok(())
    }

    /// This function is called when an app faults.
    ///
    /// This is an optional function that can be implemented by `Platform`s that
    /// allows the chip to handle the app fault and not terminate or restart
    /// the app.
    ///
    /// If `Ok(())` is returned by this function then the kernel will not
    /// terminate or restart the app, but instead allow it to continue
    /// running. NOTE in this case the chip must have fixed the underlying
    /// reason for fault otherwise it will re-occur.
    ///
    /// This can not be used for apps to circumvent Tock's protections. If
    /// for example this function just ignored the error and allowed the app
    /// to continue the fault would continue to occur.
    ///
    /// If `Err(())` is returned then the kernel will set the app as faulted
    /// and follow the `FaultResponse` protocol.
    ///
    /// It is unlikey a `Platform` will need to implement this. This should be used
    /// for only a handul of use cases. Possible use cases include:
    ///    - Allowing the kernel to emulate unimplemented instructions
    ///      This could be used to allow apps to run on hardware that doesn't
    ///      implement some instructions, for example atomics.
    ///    - Allow the kernel to handle hardware faults, triggered by the app.
    ///      This can allow an app to continue running if it triggers certain
    ///      types of faults. For example if an app triggers a memory parity
    ///      error the kernel can handle the error and allow the app to
    ///      continue (or not).
    ///    - Allow an app to execute from external QSPI.
    ///      This could be used to allow an app to execute from external QSPI
    ///      where access faults can be handled by the `Platform` to ensure the
    ///      QPSI is mapped correctly.
    #[allow(unused_variables)]
    fn process_fault_hook(&self, process: &dyn process::Process) -> Result<(), ()> {
        Err(())
    }
}
