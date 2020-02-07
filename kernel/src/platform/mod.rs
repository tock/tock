//! Interface for chips and boards.

use core::fmt::Write;
use crate::driver::Driver;
use crate::syscall;

pub mod mpu;
crate mod systick;

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
}

/// Interface for individual MCUs.
///
/// The trait defines chip-specific properties of Tock's operation. These
/// include whether and which memory protection mechanism and systick to use,
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
    type SysTick: systick::SysTick;

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

    /// Returns a reference to the implementation of the systick timer for this
    /// chip.
    fn systick(&self) -> &Self::SysTick;

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
