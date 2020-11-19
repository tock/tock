//! Interface for chips and boards.

use crate::driver::{Driver, LegacyDriver};
use crate::process;
use crate::returncode;
use crate::syscall;
use core::fmt::Write;

pub mod mpu;
pub(crate) mod scheduler_timer;
pub mod watchdog;

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
        F: FnOnce(Option<Result<&dyn Driver, &dyn LegacyDriver>>) -> R;

    /// Check the platform-provided system call filter for all non-yield system
    /// calls.  If the system call is allowed for the provided process then
    /// return Ok(()).  Otherwise, return Err with a ReturnCode that will be
    /// returned to the calling application.  The default implementation allows
    /// all system calls. This API should be considered unstable, and is likely
    /// to change in the future.
    fn filter_syscall(
        &self,
        _process: &dyn process::ProcessType,
        _syscall: &syscall::Syscall,
    ) -> Result<(), returncode::ReturnCode> {
        Ok(())
    }
}

/// Interface for individual MCUs.
///
/// The trait defines chip-specific properties of Tock's operation. These
/// include whether and which memory protection mechanism and scheduler_timer to use,
/// how to switch between the kernel and userland applications, and how to
/// handle hardware events.
///
/// Each microcontroller should define a struct and implement this trait.
pub trait Chip {
    /// The particular Memory Protection Unit (MPU) for this chip.
    type MPU: mpu::MPU;

    /// The implementation of the interface between userspace and the kernel for
    /// this specific chip. Likely this is architecture specific, but individual
    /// chips may have various custom requirements.
    type UserspaceKernelBoundary: syscall::UserspaceKernelBoundary;

    /// The implementation of the timer used to create the timeslices provided
    /// to applications.
    type SchedulerTimer: scheduler_timer::SchedulerTimer;

    /// The implementation of the WatchDog timer used to monitor the running
    /// of the kernel.
    type WatchDog: watchdog::WatchDog;

    /// The kernel calls this function to tell the chip to check for all pending
    /// interrupts and to correctly dispatch them to the peripheral drivers for
    /// the chip.
    ///
    /// This function should loop internally until all interrupts have been
    /// handled. It is ok, however, if an interrupt occurs after the last check
    /// but before this function returns. The kernel will handle this edge case.
    fn service_pending_interrupts(&self);

    /// Ask the chip to check if there are any pending interrupts.
    fn has_pending_interrupts(&self) -> bool;

    /// Returns a reference to the implementation for the MPU on this chip.
    fn mpu(&self) -> &Self::MPU;

    /// Returns a reference to the implementation of the scheduler_timer timer for this
    /// chip.
    fn scheduler_timer(&self) -> &Self::SchedulerTimer;

    /// Returns a reference to the implementation for the WatchDog on this chip.
    fn watchdog(&self) -> &Self::WatchDog;

    /// Returns a reference to the implementation for the interface between
    /// userspace and kernelspace.
    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary;

    /// Called when there is nothing left for the chip to do and it should enter
    /// a low power sleep state. This low power sleep state should allow
    /// interrupts to still be active so that the next interrupt event wakes the
    /// chip and resumes the scheduler.
    fn sleep(&self);

    /// Run a function in an atomic state, which means that interrupts are
    /// disabled so that an interrupt will not fire during the passed in
    /// function's execution.
    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R;

    /// Print out chip state (system registers) to a supplied
    /// writer. This does not print out the execution context
    /// (data registers), as this depends on how they are stored;
    /// that is implemented by
    /// `syscall::UserspaceKernelBoundary::print_context`.
    /// This also does not print out a process memory state,
    /// that is implemented by `process::Process::print_memory_map`.
    /// The MPU state is printed by the MPU's implementation of
    /// the Display trait.
    /// Used by panic.
    unsafe fn print_state(&self, writer: &mut dyn Write);
}

/// Interface for handling interrupts and deferred calls on a hardware chip.
///
/// Each board must construct an implementation of this trait to handle specific
/// interrupts. When an interrupt (identified by number) has triggered and
/// should be handled, the implementation of this trait will be called with the
/// interrupt number. The implementation can then handle the interrupt, or
/// return `false` to signify that it does not know how to handle the interrupt.
///
/// This functionality is given this `InterruptService` interface so that
/// multiple objects can be chained together to handle interrupts for a chip.
/// This is useful for code organization and removing the need for duplication
/// when multiple variations of a specific microcontroller exist. Then a shared,
/// base object can handle most interrupts, and variation-specific objects can
/// handle the variation-specific interrupts.
///
/// To simplify structuring the Rust code when using `InterruptService`, the
/// interrupt number should be passed "top-down". That is, an interrupt to be
/// handled will first be passed to the `InterruptService` object that is most
/// specific. If that object cannot handle the interrupt, then it should
/// maintain a reference to the second most specific object, and return by
/// calling to that object to handle the interrupt. This continues until the
/// base object handles the interrupt or decides that the chip does not know how
/// to handle the interrupt. For example, consider a `nRF52840` chip that
/// depends on the `nRF52` crate. If both have specific interrupts they
/// know how to handle, the flow would look like:
///
/// ```ignore
///           +---->nrf52840_peripherals
///           |        |
///           |        |
///           |        v
/// kernel-->nrf52     nrf52_peripherals
/// ```
/// where the kernel instructs the `nrf52` crate to handle interrupts, and if
/// there is an interrupt ready then that interrupt is passed through the InterruptService objects
/// until something can service it.
pub trait InterruptService<T> {
    /// Service an interrupt, if supported by this chip. If this interrupt number is not supported,
    /// return false.
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool;

    /// Service a deferred call. If this task is not supported, return false.
    unsafe fn service_deferred_call(&self, task: T) -> bool;
}

/// Generic operations that clock-like things are expected to support.
pub trait ClockInterface {
    fn is_enabled(&self) -> bool;
    fn enable(&self);
    fn disable(&self);
}

/// Helper struct for interfaces that expect clocks, but have no clock control.
pub struct NoClockControl {}
impl ClockInterface for NoClockControl {
    fn is_enabled(&self) -> bool {
        true
    }
    fn enable(&self) {}
    fn disable(&self) {}
}

/// Instance of NoClockControl for things that need references to
/// `ClockInterface` objects.
pub static mut NO_CLOCK_CONTROL: NoClockControl = NoClockControl {};
