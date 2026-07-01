// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interfaces for implementing boards in Tock.

use crate::errorcode;
use crate::platform::chip::Chip;
use crate::platform::scheduler_timer;
use crate::platform::watchdog;
use crate::process;
use crate::scheduler::Scheduler;
use crate::syscall;
use crate::syscall_driver::SyscallDriver;

/// Combination trait that boards provide to the kernel that includes all of
/// the extensible operations the kernel supports.
///
/// This is the primary method for configuring the kernel for a specific board.
pub trait KernelResources<C: Chip> {
    /// The implementation of the system call dispatch mechanism the kernel
    /// will use.
    type SyscallDriverLookup: SyscallDriverLookup;

    /// The implementation of the system call filtering mechanism the kernel
    /// will use.
    type SyscallFilter: SyscallFilter;

    /// The implementation of the process fault handling mechanism the kernel
    /// will use.
    type ProcessFault: ProcessFault;

    /// The implementation of the context switch callback handler
    /// the kernel will use.
    type ContextSwitchCallback: ContextSwitchCallback;

    /// The implementation of the scheduling algorithm the kernel will use.
    type Scheduler: Scheduler<C>;

    /// The implementation of the timer used to create the timeslices provided
    /// to applications.
    type SchedulerTimer: scheduler_timer::SchedulerTimer;

    /// The implementation of the WatchDog timer used to monitor the running
    /// of the kernel.
    type WatchDog: watchdog::WatchDog;

    /// Returns a reference to the implementation of the SyscallDriverLookup this
    /// platform will use to route syscalls.
    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup;

    /// Returns a reference to the implementation of the SyscallFilter this
    /// platform wants the kernel to use.
    fn syscall_filter(&self) -> &Self::SyscallFilter;

    /// Returns a reference to the implementation of the ProcessFault handler
    /// this platform wants the kernel to use.
    fn process_fault(&self) -> &Self::ProcessFault;

    /// Returns a reference to the implementation of the ContextSwitchCallback
    /// for this platform.
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback;

    /// Returns a reference to the implementation of the Scheduler this platform
    /// wants the kernel to use.
    fn scheduler(&self) -> &Self::Scheduler;

    /// Returns a reference to the implementation of the SchedulerTimer timer
    /// for this platform.
    fn scheduler_timer(&self) -> &Self::SchedulerTimer;

    /// Returns a reference to the implementation of the WatchDog on this
    /// platform.
    fn watchdog(&self) -> &Self::WatchDog;
}

/// Configure the system call dispatch mapping.
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
/// impl SyscallDriverLookup for Hail {
///     fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
///     where
///         F: FnOnce(Option<&dyn kernel::SyscallDriver>) -> R,
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
pub trait SyscallDriverLookup {
    /// Platform-specific mapping of syscall numbers to objects that implement
    /// the Driver methods for that syscall.
    ///
    ///
    /// An implementation
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn SyscallDriver>) -> R;
}

/// Trait for implementing system call filters that the kernel uses to decide
/// whether to handle a specific system call or not.
pub trait SyscallFilter {
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
}

/// Implement default allow all SyscallFilter trait for unit.
impl SyscallFilter for () {}

/// Trait for implementing process fault handlers to run when a process faults.
pub trait ProcessFault {
    /// This function is called when an app faults.
    ///
    /// This is an optional function that can be implemented by `Platform`s that
    /// allows the chip to handle the app fault and not terminate or restart the
    /// app.
    ///
    /// If `Ok(())` is returned by this function then the kernel will not
    /// terminate or restart the app, but instead allow it to continue running.
    /// NOTE in this case the chip must have fixed the underlying reason for
    /// fault otherwise it will re-occur.
    ///
    /// This can not be used for apps to circumvent Tock's protections. If for
    /// example this function just ignored the error and allowed the app to
    /// continue the fault would continue to occur.
    ///
    /// If `Err(())` is returned then the kernel will set the app as faulted and
    /// follow the `FaultResponse` protocol.
    ///
    /// It is unlikely a `Platform` will need to implement this. This should be
    /// used for only a handful of use cases. Possible use cases include:
    ///    - Allowing the kernel to emulate unimplemented instructions This
    ///      could be used to allow apps to run on hardware that doesn't
    ///      implement some instructions, for example atomics.
    ///    - Allow the kernel to handle hardware faults, triggered by the app.
    ///      This can allow an app to continue running if it triggers certain
    ///      types of faults. For example if an app triggers a memory parity
    ///      error the kernel can handle the error and allow the app to continue
    ///      (or not).
    ///    - Allow an app to execute from external QSPI. This could be used to
    ///      allow an app to execute from external QSPI where access faults can
    ///      be handled by the `Platform` to ensure the QPSI is mapped
    ///      correctly.
    #[allow(unused_variables)]
    fn process_fault_hook(&self, process: &dyn process::Process) -> Result<(), ()> {
        Err(())
    }
}

/// Implement default ProcessFault trait for unit.
impl ProcessFault for () {}

/// Trait for implementing handlers on userspace context switches.
pub trait ContextSwitchCallback {
    /// This function is called before the kernel switches to a process.
    ///
    /// `process` is the app that is about to run
    fn context_switch_hook(&self, process: &dyn process::Process);
}

/// Implement default ContextSwitchCallback trait for unit.
impl ContextSwitchCallback for () {
    fn context_switch_hook(&self, _process: &dyn process::Process) {}
}

/// Trait for inspecting or modifying upcall arguments before they are
/// delivered to a process.
///
/// The kernel calls [`on_upcall`](UpcallVerifier::on_upcall) for every
/// [`crate::process::Task::FunctionCall`] sourced from a driver
/// (`FunctionCallSource::Driver`) just before the arguments are committed to
/// the process's register file. Kernel-sourced function calls (e.g. the
/// process init function) bypass this hook.
///
/// The default no-op implementation is `impl UpcallVerifier for ()`. Boards
/// install a custom implementation via
/// [`crate::kernel::Kernel::register_upcall_verifier`].
pub trait UpcallVerifier {
    /// Called before an upcall is delivered to a process.
    ///
    /// `upcall_id` identifies the driver/subscribe pair. `r0`–`r2` are the
    /// upcall arguments the capsule scheduled via `schedule_upcall`.
    ///
    /// Return [`UpcallAction::Proceed`] to deliver the upcall unchanged, or
    /// [`UpcallAction::Overwrite`] to substitute different argument values
    /// (e.g. to normalise a live timestamp that differs between two harts in a
    /// lockstep configuration).
    fn on_upcall(
        &self,
        upcall_id: crate::upcall::UpcallId,
        r0: usize,
        r1: usize,
        r2: usize,
    ) -> UpcallAction;
}

/// Action returned by [`UpcallVerifier::on_upcall`].
#[derive(Copy, Clone)]
pub enum UpcallAction {
    /// Deliver the upcall with its original arguments unchanged.
    Proceed,
    /// Deliver the upcall with the provided argument values instead.
    Overwrite { r0: usize, r1: usize, r2: usize },
}

/// No-op [`UpcallVerifier`]: all upcalls are delivered unchanged.
impl UpcallVerifier for () {
    #[inline(always)]
    fn on_upcall(
        &self,
        _upcall_id: crate::upcall::UpcallId,
        _r0: usize,
        _r1: usize,
        _r2: usize,
    ) -> UpcallAction {
        UpcallAction::Proceed
    }
}
