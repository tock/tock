//! Interface for chips and boards.

use crate::driver::Driver;
use crate::errorcode;
use crate::process;
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

    /// The description of a core
    /// For now this is an opaque type used by the implementation
    type Core;

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

    /// Get the number of available cores
    /// The default implementation returns 1 as most of
    /// the MCUs have only one available core
    #[inline(always)]
    fn core_count(&self) -> usize {
        1
    }

    /// Get the current Core
    /// The actual data type depends on the implementation
    fn current_core(&self) -> &Self::Core;

    /// Get the number of available spinlocks
    /// The default implemenation returns 1 spinlock
    /// so that this function may be transparently used
    /// by the kernel
    #[inline(always)]
    fn spinlock_count() -> usize {
        1
    }

    /// Execute on a single core at the same time
    /// As different MCUs provide different number hardware spinlock,
    /// the spinlock_id is a hint about what spinlock number to use
    /// A simple algorithm would be using the remainder
    ///   actual_spinlock = spinlock_id % self.spinlock_count
    /// The default implementation panics if there are more than 1 cores avialable
    /// otherwise simple executes the provided function
    #[allow(unused_variables)]
    #[inline(always)]
    fn spinlock<F, R>(&self, spinlock_id: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if self.core_count() > 1 {
            panic!("no spinlocks available");
        } else {
            f()
        }
    }

    /// Start a core and use a function for its entry point
    /// The stack frame setup is left up to the implementation
    /// The function that is run on the core recevises two arguments:
    ///   a pointer to the stack bottom
    ///   the length of the stack
    /// The function that is ran on the core should never return
    #[allow(unused_variables)]
    unsafe fn start_core<F>(&self, core: &Self::Core, f: F) -> Result<(), ()>
    where
        F: Fn(*const u8, usize) -> !,
    {
        Err(())
    }
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
