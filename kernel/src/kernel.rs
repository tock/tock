//! Tock's main kernel loop, scheduler loop, and Scheduler trait.
//!
//! This module also includes utility functions that are commonly used
//! by scheduler policy implementations.  Scheduling policy (round
//! robin, priority, etc.) is defined in the `sched` subcrate and
//! selected by a board.

use core::cell::Cell;
use core::ptr::NonNull;

use crate::capabilities;
use crate::config;
use crate::debug;
use crate::dynamic_deferred_call::DynamicDeferredCall;
use crate::errorcode::ErrorCode;
use crate::grant::Grant;
use crate::ipc;
use crate::memop;
use crate::platform::chip::Chip;
use crate::platform::mpu::MPU;
use crate::platform::platform::KernelResources;
use crate::platform::platform::{ProcessFault, SyscallDispatch, SyscallFilter};
use crate::platform::scheduler_timer::SchedulerTimer;
use crate::platform::watchdog::WatchDog;
use crate::process::ProcessId;
use crate::process::{self, Task};
use crate::scheduler::{Scheduler, SchedulingDecision};
use crate::syscall::{ContextSwitchReason, SyscallReturn};
use crate::syscall::{Syscall, YieldCall};
use crate::syscall_driver::CommandReturn;
use crate::upcall::{Upcall, UpcallId};
use crate::utilities::cells::NumericCellExt;

/// Threshold in microseconds to consider a process's timeslice to be exhausted.
/// That is, Tock will skip re-scheduling a process if its remaining timeslice
/// is less than this threshold.
pub(crate) const MIN_QUANTA_THRESHOLD_US: u32 = 500;

/// Main object for the kernel. Each board will need to create one.
pub struct Kernel {
    /// How many "to-do" items exist at any given time. These include
    /// outstanding upcalls and processes in the Running state.
    work: Cell<usize>,

    /// This holds a pointer to the static array of Process pointers.
    processes: &'static [Option<&'static dyn process::Process>],

    /// A counter which keeps track of how many process identifiers have been
    /// created. This is used to create new unique identifiers for processes.
    process_identifier_max: Cell<usize>,

    /// How many grant regions have been setup. This is incremented on every
    /// call to `create_grant()`. We need to explicitly track this so that when
    /// processes are created they can be allocated pointers for each grant.
    grant_counter: Cell<usize>,

    /// Flag to mark that grants have been finalized. This means that the kernel
    /// cannot support creating new grants because processes have already been
    /// created and the data structures for grants have already been
    /// established.
    grants_finalized: Cell<bool>,
}

/// Enum used to inform scheduler why a process stopped executing (aka why
/// `do_process()` returned).
#[derive(PartialEq, Eq)]
pub enum StoppedExecutingReason {
    /// The process returned because it is no longer ready to run.
    NoWorkLeft,

    /// The process faulted, and the board restart policy was configured such
    /// that it was not restarted and there was not a kernel panic.
    StoppedFaulted,

    /// The kernel stopped the process.
    Stopped,

    /// The process was preempted because its timeslice expired.
    TimesliceExpired,

    /// The process returned because it was preempted by the kernel. This can
    /// mean that kernel work became ready (most likely because an interrupt
    /// fired and the kernel thread needs to execute the bottom half of the
    /// interrupt), or because the scheduler no longer wants to execute that
    /// process.
    KernelPreemption,
}

impl Kernel {
    pub fn new(processes: &'static [Option<&'static dyn process::Process>]) -> Kernel {
        Kernel {
            work: Cell::new(0),
            processes,
            process_identifier_max: Cell::new(0),
            grant_counter: Cell::new(0),
            grants_finalized: Cell::new(false),
        }
    }

    /// Something was scheduled for a process, so there is more work to do.
    ///
    /// This is only exposed in the core kernel crate.
    pub(crate) fn increment_work(&self) {
        self.work.increment();
    }

    /// Something was scheduled for a process, so there is more work to do.
    ///
    /// This is exposed publicly, but restricted with a capability. The intent
    /// is that external implementations of `Process` need to be able to
    /// indicate there is more process work to do.
    pub fn increment_work_external(
        &self,
        _capability: &dyn capabilities::ExternalProcessCapability,
    ) {
        self.increment_work();
    }

    /// Something finished for a process, so we decrement how much work there is
    /// to do.
    ///
    /// This is only exposed in the core kernel crate.
    pub(crate) fn decrement_work(&self) {
        self.work.decrement();
    }

    /// Something finished for a process, so we decrement how much work there is
    /// to do.
    ///
    /// This is exposed publicly, but restricted with a capability. The intent
    /// is that external implementations of `Process` need to be able to
    /// indicate that some process work has finished.
    pub fn decrement_work_external(
        &self,
        _capability: &dyn capabilities::ExternalProcessCapability,
    ) {
        self.decrement_work();
    }

    /// Helper function for determining if we should service processes or go to
    /// sleep.
    pub(crate) fn processes_blocked(&self) -> bool {
        self.work.get() == 0
    }

    /// Helper function that moves all non-generic portions of process_map_or
    /// into a non-generic function to reduce code bloat from monomorphization.
    pub(crate) fn get_process(&self, processid: ProcessId) -> Option<&dyn process::Process> {
        // We use the index in the `appid` so we can do a direct lookup.
        // However, we are not guaranteed that the app still exists at that
        // index in the processes array. To avoid additional overhead, we do the
        // lookup and check here, rather than calling `.index()`.
        match self.processes.get(processid.index) {
            Some(Some(process)) => {
                // Check that the process stored here matches the identifier
                // in the `appid`.
                if process.processid() == processid {
                    Some(*process)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Run a closure on a specific process if it exists. If the process with a
    /// matching `ProcessId` does not exist at the index specified within the
    /// `ProcessId`, then `default` will be returned.
    ///
    /// A match will not be found if the process was removed (and there is a
    /// `None` in the process array), if the process changed its identifier
    /// (likely after being restarted), or if the process was moved to a
    /// different index in the processes array. Note that a match _will_ be
    /// found if the process still exists in the correct location in the array
    /// but is in any "stopped" state.
    pub(crate) fn process_map_or<F, R>(&self, default: R, appid: ProcessId, closure: F) -> R
    where
        F: FnOnce(&dyn process::Process) -> R,
    {
        match self.get_process(appid) {
            Some(process) => closure(process),
            None => default,
        }
    }

    /// Run a closure on every valid process. This will iterate the array of
    /// processes and call the closure on every process that exists.
    pub(crate) fn process_each<F>(&self, closure: F)
    where
        F: Fn(&dyn process::Process),
    {
        for process in self.processes.iter() {
            match process {
                Some(p) => {
                    closure(*p);
                }
                None => {}
            }
        }
    }

    /// Returns an iterator over all processes loaded by the kernel
    pub(crate) fn get_process_iter(
        &self,
    ) -> core::iter::FilterMap<
        core::slice::Iter<Option<&dyn process::Process>>,
        fn(&Option<&'static dyn process::Process>) -> Option<&'static dyn process::Process>,
    > {
        fn keep_some(
            &x: &Option<&'static dyn process::Process>,
        ) -> Option<&'static dyn process::Process> {
            x
        }
        self.processes.iter().filter_map(keep_some)
    }

    /// Run a closure on every valid process. This will iterate the array of
    /// processes and call the closure on every process that exists.
    ///
    /// This is functionally the same as `process_each()`, but this method is
    /// available outside the kernel crate and requires a
    /// `ProcessManagementCapability` to use.
    pub fn process_each_capability<F>(
        &'static self,
        _capability: &dyn capabilities::ProcessManagementCapability,
        closure: F,
    ) where
        F: Fn(&dyn process::Process),
    {
        for process in self.processes.iter() {
            match process {
                Some(p) => {
                    closure(*p);
                }
                None => {}
            }
        }
    }

    /// Run a closure on every process, but only continue if the closure returns `None`. That is,
    /// if the closure returns any non-`None` value, iteration stops and the value is returned from
    /// this function to the called.
    pub(crate) fn process_until<T, F>(&self, closure: F) -> Option<T>
    where
        F: Fn(&dyn process::Process) -> Option<T>,
    {
        for process in self.processes.iter() {
            match process {
                Some(p) => {
                    let ret = closure(*p);
                    if ret.is_some() {
                        return ret;
                    }
                }
                None => {}
            }
        }
        None
    }

    /// Retrieve the `ProcessId` of the given app based on its identifier. This is
    /// useful if an app identifier is passed to the kernel from somewhere (such
    /// as from userspace) and needs to be expanded to a full `ProcessId` for use
    /// with other APIs.
    pub(crate) fn lookup_app_by_identifier(&self, identifier: usize) -> Option<ProcessId> {
        self.processes.iter().find_map(|&p| {
            p.map_or(None, |p2| {
                if p2.processid().id() == identifier {
                    Some(p2.processid())
                } else {
                    None
                }
            })
        })
    }

    /// Checks if the provided `ProcessId` is still valid given the processes stored
    /// in the processes array. Returns `true` if the ProcessId still refers to
    /// a valid process, and `false` if not.
    ///
    /// This is needed for `ProcessId` itself to implement the `.index()` command to
    /// verify that the referenced app is still at the correct index.
    pub(crate) fn processid_is_valid(&self, appid: &ProcessId) -> bool {
        self.processes.get(appid.index).map_or(false, |p| {
            p.map_or(false, |process| process.processid().id() == appid.id())
        })
    }

    /// Create a new grant. This is used in board initialization to setup grants
    /// that capsules use to interact with processes.
    ///
    /// Grants **must** only be created _before_ processes are initialized.
    /// Processes use the number of grants that have been allocated to correctly
    /// initialize the process's memory with a pointer for each grant. If a
    /// grant is created after processes are initialized this will panic.
    ///
    /// Calling this function is restricted to only certain users, and to
    /// enforce this calling this function requires the
    /// `MemoryAllocationCapability` capability.
    pub fn create_grant<T: Default, const NUM_UPCALLS: usize>(
        &'static self,
        driver_num: usize,
        _capability: &dyn capabilities::MemoryAllocationCapability,
    ) -> Grant<T, NUM_UPCALLS> {
        if self.grants_finalized.get() {
            panic!("Grants finalized. Cannot create a new grant.");
        }

        // Create and return a new grant.
        let grant_index = self.grant_counter.get();
        self.grant_counter.increment();
        Grant::new(self, driver_num, grant_index)
    }

    /// Returns the number of grants that have been setup in the system and
    /// marks the grants as "finalized". This means that no more grants can
    /// be created because data structures have been setup based on the number
    /// of grants when this function is called.
    ///
    /// In practice, this is called when processes are created, and the process
    /// memory is setup based on the number of current grants.
    pub(crate) fn get_grant_count_and_finalize(&self) -> usize {
        self.grants_finalized.set(true);
        self.grant_counter.get()
    }

    /// Returns the number of grants that have been setup in the system and
    /// marks the grants as "finalized". This means that no more grants can
    /// be created because data structures have been setup based on the number
    /// of grants when this function is called.
    ///
    /// In practice, this is called when processes are created, and the process
    /// memory is setup based on the number of current grants.
    ///
    /// This is exposed publicly, but restricted with a capability. The intent
    /// is that external implementations of `Process` need to be able to
    /// retrieve the final number of grants.
    pub fn get_grant_count_and_finalize_external(
        &self,
        _capability: &dyn capabilities::ExternalProcessCapability,
    ) -> usize {
        self.get_grant_count_and_finalize()
    }

    /// Create a new unique identifier for a process and return the identifier.
    ///
    /// Typically we just choose a larger number than we have used for any process
    /// before which ensures that the identifier is unique.
    pub(crate) fn create_process_identifier(&self) -> usize {
        self.process_identifier_max.get_and_increment()
    }

    /// Cause all apps to fault.
    ///
    /// This will call `set_fault_state()` on each app, causing the app to enter
    /// the state as if it had crashed (for example with an MPU violation). If
    /// the process is configured to be restarted it will be.
    ///
    /// Only callers with the `ProcessManagementCapability` can call this
    /// function. This restricts general capsules from being able to call this
    /// function, since capsules should not be able to arbitrarily restart all
    /// apps.
    pub fn hardfault_all_apps<C: capabilities::ProcessManagementCapability>(&self, _c: &C) {
        for p in self.processes.iter() {
            p.map(|process| {
                process.set_fault_state();
            });
        }
    }

    /// Perform one iteration of the core Tock kernel loop.
    ///
    /// This function is responsible for three main operations:
    ///
    /// 1. Check if the kernel itself has any work to be done and if
    /// the scheduler wants to complete that work now. If so, it allows
    /// the kernel to run.
    /// 2. Check if any processes have any work to be done, and if so
    /// if the scheduler wants to allow any processes to run now, and if so
    /// which one.
    /// 3. After ensuring the scheduler does not want to complete any kernel
    /// or process work (or there is no work to be done), are there are no outstanding
    /// interrupts to handle, put the chip to sleep.
    ///
    /// This function has one configuration option: `no_sleep`. If that
    /// argument is set to true, the kernel will never attempt to put the
    /// chip to sleep, and this function can be called again immediately.
    pub fn kernel_loop_operation<
        KR: KernelResources<C>,
        C: Chip,
        const NUM_PROCS: usize,
        const NUM_UPCALLS_IPC: usize,
    >(
        &self,
        resources: &KR,
        chip: &C,
        ipc: Option<&ipc::IPC<NUM_PROCS, NUM_UPCALLS_IPC>>,
        no_sleep: bool,
        _capability: &dyn capabilities::MainLoopCapability,
    ) {
        let scheduler = resources.scheduler();

        resources.watchdog().tickle();
        unsafe {
            // Ask the scheduler if we should do tasks inside of the kernel,
            // such as handle interrupts. A scheduler may want to prioritize
            // processes instead, or there may be no kernel work to do.
            match scheduler.do_kernel_work_now(chip) {
                true => {
                    // Execute kernel work. This includes handling
                    // interrupts and is how code in the chips/ and capsules
                    // crates is able to execute.
                    scheduler.execute_kernel_work(chip);
                }
                false => {
                    // No kernel work ready, so ask scheduler for a process.
                    match scheduler.next(self) {
                        SchedulingDecision::RunProcess((appid, timeslice_us)) => {
                            self.process_map_or((), appid, |process| {
                                let (reason, time_executed) =
                                    self.do_process(resources, chip, process, ipc, timeslice_us);
                                scheduler.result(reason, time_executed);
                            });
                        }
                        SchedulingDecision::TrySleep => {
                            // For testing, it may be helpful to
                            // disable sleeping the chip in case
                            // the running test does not generate
                            // any interrupts.
                            if !no_sleep {
                                chip.atomic(|| {
                                    // Cannot sleep if interrupts are pending,
                                    // as on most platforms unhandled interrupts
                                    // will wake the device. Also, if the only
                                    // pending interrupt occurred after the
                                    // scheduler decided to put the chip to
                                    // sleep, but before this atomic section
                                    // starts, the interrupt will not be
                                    // serviced and the chip will never wake
                                    // from sleep.
                                    if !chip.has_pending_interrupts()
                                        && !DynamicDeferredCall::global_instance_calls_pending()
                                            .unwrap_or(false)
                                    {
                                        resources.watchdog().suspend();
                                        chip.sleep();
                                        resources.watchdog().resume();
                                    }
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    /// Main loop of the OS.
    ///
    /// Most of the behavior of this loop is controlled by the `Scheduler`
    /// implementation in use.
    pub fn kernel_loop<
        KR: KernelResources<C>,
        C: Chip,
        const NUM_PROCS: usize,
        const NUM_UPCALLS_IPC: usize,
    >(
        &self,
        resources: &KR,
        chip: &C,
        ipc: Option<&ipc::IPC<NUM_PROCS, NUM_UPCALLS_IPC>>,
        capability: &dyn capabilities::MainLoopCapability,
    ) -> ! {
        resources.watchdog().setup();
        loop {
            self.kernel_loop_operation(resources, chip, ipc, false, capability);
        }
    }

    /// Transfer control from the kernel to a userspace process.
    ///
    /// This function is called by the main kernel loop to run userspace code.
    /// Notably, system calls from processes are handled in the kernel, *by the
    /// kernel thread* in this function, and the syscall return value is set for
    /// the process immediately. Normally, a process is allowed to continue
    /// running after calling a syscall. However, the scheduler is given an out,
    /// as `do_process()` will check with the scheduler before re-executing the
    /// process to allow it to return from the syscall. If a process yields with
    /// no upcalls pending, exits, exceeds its timeslice, or is interrupted,
    /// then `do_process()` will return.
    ///
    /// Depending on the particular scheduler in use, this function may act in a
    /// few different ways. `scheduler.continue_process()` allows the scheduler
    /// to tell the Kernel whether to continue executing the process, or to
    /// return control to the scheduler as soon as a kernel task becomes ready
    /// (either a bottom half interrupt handler or dynamic deferred call), or to
    /// continue executing the userspace process until it reaches one of the
    /// aforementioned stopping conditions. Some schedulers may not require a
    /// scheduler timer; passing `None` for the timeslice will use a null
    /// scheduler timer even if the chip provides a real scheduler timer.
    /// Schedulers can pass a timeslice (in us) of their choice, though if the
    /// passed timeslice is smaller than `MIN_QUANTA_THRESHOLD_US` the process
    /// will not execute, and this function will return immediately.
    ///
    /// This function returns a tuple indicating the reason the reason this
    /// function has returned to the scheduler, and the amount of time the
    /// process spent executing (or `None` if the process was run
    /// cooperatively). Notably, time spent in this function by the kernel,
    /// executing system calls or merely setting up the switch to/from
    /// userspace, is charged to the process.
    fn do_process<
        KR: KernelResources<C>,
        C: Chip,
        const NUM_PROCS: usize,
        const NUM_UPCALLS_IPC: usize,
    >(
        &self,
        resources: &KR,
        chip: &C,
        process: &dyn process::Process,
        ipc: Option<&crate::ipc::IPC<NUM_PROCS, NUM_UPCALLS_IPC>>,
        timeslice_us: Option<u32>,
    ) -> (StoppedExecutingReason, Option<u32>) {
        // We must use a dummy scheduler timer if the process should be executed
        // without any timeslice restrictions. Note, a chip may not provide a
        // real scheduler timer implementation even if a timeslice is requested.
        let scheduler_timer: &dyn SchedulerTimer = if timeslice_us.is_none() {
            &() // dummy timer, no preemption
        } else {
            resources.scheduler_timer()
        };

        // Clear the scheduler timer and then start the counter. This starts the
        // process's timeslice. Since the kernel is still executing at this
        // point, the scheduler timer need not have an interrupt enabled after
        // `start()`.
        scheduler_timer.reset();
        timeslice_us.map(|timeslice| scheduler_timer.start(timeslice));

        // Need to track why the process is no longer executing so that we can
        // inform the scheduler.
        let mut return_reason = StoppedExecutingReason::NoWorkLeft;

        // Since the timeslice counts both the process's execution time and the
        // time spent in the kernel on behalf of the process (setting it up and
        // handling its syscalls), we intend to keep running the process until
        // it has no more work to do. We break out of this loop if the scheduler
        // no longer wants to execute this process or if it exceeds its
        // timeslice.
        loop {
            let stop_running = match scheduler_timer.get_remaining_us() {
                Some(us) => us <= MIN_QUANTA_THRESHOLD_US,
                None => true,
            };
            if stop_running {
                // Process ran out of time while the kernel was executing.
                process.debug_timeslice_expired();
                return_reason = StoppedExecutingReason::TimesliceExpired;
                break;
            }

            // Check if the scheduler wishes to continue running this process.
            let continue_process = unsafe {
                resources
                    .scheduler()
                    .continue_process(process.processid(), chip)
            };
            if !continue_process {
                return_reason = StoppedExecutingReason::KernelPreemption;
                break;
            }

            // Check if this process is actually ready to run. If not, we don't
            // try to run it. This case can happen if a process faults and is
            // stopped, for example.
            if !process.ready() {
                return_reason = StoppedExecutingReason::NoWorkLeft;
                break;
            }

            match process.get_state() {
                process::State::Running => {
                    // Running means that this process expects to be running, so
                    // go ahead and set things up and switch to executing the
                    // process. Arming the scheduler timer instructs it to
                    // generate an interrupt when the timeslice has expired. The
                    // underlying timer is not affected.
                    process.setup_mpu();

                    chip.mpu().enable_app_mpu();
                    scheduler_timer.arm();
                    let context_switch_reason = process.switch_to();
                    scheduler_timer.disarm();
                    chip.mpu().disable_app_mpu();

                    // Now the process has returned back to the kernel. Check
                    // why and handle the process as appropriate.
                    match context_switch_reason {
                        Some(ContextSwitchReason::Fault) => {
                            // The app faulted, check if the chip wants to
                            // handle the fault.
                            if resources
                                .process_fault()
                                .process_fault_hook(process)
                                .is_err()
                            {
                                // Let process deal with it as appropriate.
                                process.set_fault_state();
                            }
                        }
                        Some(ContextSwitchReason::SyscallFired { syscall }) => {
                            self.handle_syscall(resources, process, syscall);
                        }
                        Some(ContextSwitchReason::Interrupted) => {
                            if scheduler_timer.get_remaining_us().is_none() {
                                // This interrupt was a timeslice expiration.
                                process.debug_timeslice_expired();
                                return_reason = StoppedExecutingReason::TimesliceExpired;
                                break;
                            }
                            // Go to the beginning of loop to determine whether
                            // to break to handle the interrupt, continue
                            // executing this process, or switch to another
                            // process.
                            continue;
                        }
                        None => {
                            // Something went wrong when switching to this
                            // process. Indicate this by putting it in a fault
                            // state.
                            process.set_fault_state();
                        }
                    }
                }
                process::State::Yielded | process::State::Unstarted => {
                    // If the process is yielded or hasn't been started it is
                    // waiting for a upcall. If there is a task scheduled for
                    // this process go ahead and set the process to execute it.
                    match process.dequeue_task() {
                        None => break,
                        Some(cb) => match cb {
                            Task::FunctionCall(ccb) => {
                                if config::CONFIG.trace_syscalls {
                                    debug!(
                                        "[{:?}] function_call @{:#x}({:#x}, {:#x}, {:#x}, {:#x})",
                                        process.processid(),
                                        ccb.pc,
                                        ccb.argument0,
                                        ccb.argument1,
                                        ccb.argument2,
                                        ccb.argument3,
                                    );
                                }
                                process.set_process_function(ccb);
                            }
                            Task::IPC((otherapp, ipc_type)) => {
                                ipc.map_or_else(
                                    || {
                                        assert!(
                                            false,
                                            "Kernel consistency error: IPC Task with no IPC"
                                        );
                                    },
                                    |ipc| {
                                        // TODO(alevy): this could error for a variety of reasons.
                                        // Should we communicate the error somehow?
                                        // https://github.com/tock/tock/issues/1993
                                        unsafe {
                                            let _ = ipc.schedule_upcall(
                                                process.processid(),
                                                otherapp,
                                                ipc_type,
                                            );
                                        }
                                    },
                                );
                            }
                        },
                    }
                }
                process::State::Faulted | process::State::Terminated => {
                    // We should never be scheduling a process in fault.
                    panic!("Attempted to schedule a faulty process");
                }
                process::State::StoppedRunning => {
                    return_reason = StoppedExecutingReason::Stopped;
                    break;
                }
                process::State::StoppedYielded => {
                    return_reason = StoppedExecutingReason::Stopped;
                    break;
                }
            }
        }

        // Check how much time the process used while it was executing, and
        // return the value so we can provide it to the scheduler.
        let time_executed_us = timeslice_us.map_or(None, |timeslice| {
            // Note, we cannot call `.get_remaining_us()` again if it has previously
            // returned `None`, so we _must_ check the return reason first.
            if return_reason == StoppedExecutingReason::TimesliceExpired {
                // used the whole timeslice
                Some(timeslice)
            } else {
                match scheduler_timer.get_remaining_us() {
                    Some(remaining) => Some(timeslice - remaining),
                    None => Some(timeslice), // used whole timeslice
                }
            }
        });

        // Reset the scheduler timer in case it unconditionally triggers
        // interrupts upon expiration. We do not want it to expire while the
        // chip is sleeping, for example.
        scheduler_timer.reset();

        (return_reason, time_executed_us)
    }

    /// Method to invoke a system call on a particular process.
    /// Applies the kernel system call filtering policy (if any).
    /// Handles `Yield` and `Exit`, dispatches `Memop` to `memop::memop`,
    /// and dispatches peripheral driver system calls to peripheral
    /// driver capsules through the platforms `with_driver` method.
    #[inline]
    fn handle_syscall<KR: KernelResources<C>, C: Chip>(
        &self,
        resources: &KR,
        process: &dyn process::Process,
        syscall: Syscall,
    ) {
        // Hook for process debugging.
        process.debug_syscall_called(syscall);

        // Enforce platform-specific syscall filtering here.
        //
        // Before continuing to handle non-yield syscalls
        // the kernel first checks if the platform wants to
        // block that syscall for the process, and if it
        // does, sets a return value which is returned to
        // the calling process.
        //
        // Filtering a syscall (i.e. blocking the syscall
        // from running) does not cause the process to loose
        // its timeslice. The error will be returned
        // immediately (assuming the process has not already
        // exhausted its timeslice) allowing the process to
        // decide how to handle the error.
        match syscall {
            Syscall::Yield {
                which: _,
                address: _,
            } => {} // Yield is not filterable
            Syscall::Exit {
                which: _,
                completion_code: _,
            } => {} // Exit is not filterable
            Syscall::Memop {
                operand: _,
                arg0: _,
            } => {} // Memop is not filterable
            _ => {
                // Check all other syscalls for filtering
                if let Err(response) = resources.syscall_filter().filter_syscall(process, &syscall)
                {
                    process.set_syscall_return_value(SyscallReturn::Failure(response));

                    return;
                }
            }
        }

        // Handle each of the syscalls.
        match syscall {
            Syscall::Memop { operand, arg0 } => {
                let rval = memop::memop(process, operand, arg0);
                if config::CONFIG.trace_syscalls {
                    debug!(
                        "[{:?}] memop({}, {:#x}) = {:?}",
                        process.processid(),
                        operand,
                        arg0,
                        rval
                    );
                }
                process.set_syscall_return_value(rval);
            }
            Syscall::Yield { which, address } => {
                if config::CONFIG.trace_syscalls {
                    debug!("[{:?}] yield. which: {}", process.processid(), which);
                }
                if which > (YieldCall::Wait as usize) {
                    // Only 0 and 1 are valid, so this is not a valid
                    // yield system call, Yield does not have a return
                    // value because it can push a function call onto
                    // the stack; just return control to the process.
                    return;
                }
                let wait = which == (YieldCall::Wait as usize);
                // If this is a yield-no-wait AND there are no pending
                // tasks, then return immediately. Otherwise, go into the
                // yielded state and execute tasks now or when they arrive.
                let return_now = !wait && !process.has_tasks();
                if return_now {
                    // Set the "did I trigger upcalls" flag to be 0,
                    // return immediately. If address is invalid does
                    // nothing.
                    //
                    // Safety: this is fine as long as no references to the
                    // process's memory exist. We do not have a reference,
                    // so we can safely call `set_byte()`.
                    unsafe {
                        process.set_byte(address, 0);
                    }
                } else {
                    // There are already enqueued upcalls to execute or
                    // we should wait for them: handle in the next loop
                    // iteration and set the "did I trigger upcalls" flag
                    // to be 1. If address is invalid does nothing.
                    //
                    // Safety: this is fine as long as no references to the
                    // process's memory exist. We do not have a reference,
                    // so we can safely call `set_byte()`.
                    unsafe {
                        process.set_byte(address, 1);
                    }
                    process.set_yielded_state();
                }
            }
            Syscall::Subscribe {
                driver_number,
                subdriver_number,
                upcall_ptr,
                appdata,
            } => {
                // A upcall is identified as a tuple of the driver number and
                // the subdriver number.
                let upcall_id = UpcallId {
                    driver_num: driver_number,
                    subscribe_num: subdriver_number,
                };

                // First check if `upcall_ptr` is null. A null `upcall_ptr` will
                // result in `None` here and represents the special
                // "unsubscribe" operation.
                let ptr = NonNull::new(upcall_ptr);

                // For convenience create an `Upcall` type now. This is just a
                // data structure and doesn't do any checking or conversion.
                let upcall = Upcall::new(process.processid(), upcall_id, appdata, ptr);

                // If `ptr` is not null, we must first verify that the upcall
                // function pointer is within process accessible memory. Per
                // TRD104:
                //
                // > If the passed upcall is not valid (is outside process
                // > executable memory...), the kernel...MUST immediately return
                // > a failure with a error code of `INVALID`.
                let rval1 = ptr.map_or(None, |upcall_ptr_nonnull| {
                    if !process.is_valid_upcall_function_pointer(upcall_ptr_nonnull) {
                        Some(ErrorCode::INVAL)
                    } else {
                        None
                    }
                });

                // If the upcall is either null or valid, then we continue
                // handling the upcall.
                let rval = match rval1 {
                    Some(err) => upcall.into_subscribe_failure(err),
                    None => {
                        // At this point we must save the new upcall and return the
                        // old. The upcalls are stored by the core kernel in the
                        // grant region so we can guarantee a correct upcall swap.
                        // However, we do need help with initially allocating the
                        // grant if this driver has never been used before.
                        //
                        // To avoid the overhead with checking for process liveness
                        // and grant allocation, we assume the grant is initially
                        // allocated. If it turns out it isn't we ask the capsule to
                        // allocate the grant.
                        match crate::grant::subscribe(process, upcall) {
                            Ok(old_upcall) => old_upcall.into_subscribe_success(),
                            Err((new_upcall, _err)) => {
                                // If we get here the subscribe swap failed. We try
                                // again after asking the capsule to allocate the
                                // grant.
                                resources
                                .syscall_dispatch()
                                .with_driver(driver_number, |driver| match driver {
                                    Some(d) => {
                                        // For debugging purposes, query the
                                        // number of allocated grants of the
                                        // process. This can be used to
                                        // determine whether the driver has
                                        // allocated its Grant region correctly
                                        // as requested. If the process is not
                                        // active, we use 0 as a default value.
                                        // This should never happen on a
                                        // subscribe system call.
                                        let allocated_grants_count = if config::CONFIG.trace_syscalls {
                                            process.grant_allocated_count().unwrap_or(0)
                                        } else {
                                            0
                                        };

                                        if d.allocate_grant(process.processid()).is_err() {
                                            // If the capsule errors on allocation
                                            // we assume it is because the grant
                                            // could not be created and return an
                                            // error to userspace.
                                            new_upcall.into_subscribe_failure(ErrorCode::NOMEM)
                                        } else {
                                            // Now we try again. It is possible that
                                            // the capsule did not actually allocate
                                            // the grant, at which point this will
                                            // fail again and we return an error to
                                            // userspace.
                                            match crate::grant::subscribe(process, new_upcall) {
                                                // An Ok() returns the previous upcall,
                                                // while Err() returns the one that was
                                                // just passed.
                                                Ok(old_upcall) => {
                                                    old_upcall.into_subscribe_success()
                                                }
                                                Err((new_upcall, ErrorCode::NOMEM)) => {
                                                    // Special case the handling of ErrorCode::NOMEM,
                                                    // as it indicates a missing Grant allocation.
                                                    // Depending on the kernel configuration, we
                                                    // inform the user about the root cause of this
                                                    // issue.
                                                    if config::CONFIG.trace_syscalls {
                                                        // It appears the Grant is still not allocated.
                                                        // Based on whether the number of allocated
                                                        // Grants, we can determine whether the driver
                                                        // has allocated an additional, unrelated Grant
                                                        // region (with a different driver_num), or
                                                        // none at all. Inform the developer accordingly.
                                                        if let Some(currently_allocated_grants_count) =
                                                            process.grant_allocated_count() {
                                                                if currently_allocated_grants_count
                                                                    > allocated_grants_count
                                                                {
                                                                    debug!("[{:?}] ERROR driver {} allocated wrong grant for upcalls",
                                                                           process.processid(), driver_number);
                                                                } else {
                                                                    debug!("[{:?}] WARN driver {} did not allocate grant for upcalls",
                                                                           process.processid(), driver_number);
                                                                }
                                                            }
                                                    }
                                                    new_upcall.into_subscribe_failure(ErrorCode::NOMEM)
                                                }
                                                Err((new_upcall, err)) => {
                                                    // Handler for all errors other than
                                                    // ErrorCode::NOMEM, for example when
                                                    // the subscribe number exceeds the
                                                    // number of Upcalls in the Grant region.
                                                    new_upcall.into_subscribe_failure(err)
                                                }
                                            }
                                        }
                                    }
                                    None => new_upcall.into_subscribe_failure(ErrorCode::NODEVICE),
                                })
                            }
                        }
                    }
                };

                // Per TRD104, we only clear upcalls if the subscribe will
                // return success. At this point we know the result and clear if
                // necessary.
                if rval.is_success() {
                    // Only one upcall should exist per tuple. To ensure that
                    // there are no pending upcalls with the same identifier but
                    // with the old function pointer, we clear them now.
                    process.remove_pending_upcalls(upcall_id);
                }

                if config::CONFIG.trace_syscalls {
                    debug!(
                        "[{:?}] subscribe({:#x}, {}, @{:#x}, {:#x}) = {:?}",
                        process.processid(),
                        driver_number,
                        subdriver_number,
                        upcall_ptr as usize,
                        appdata,
                        rval
                    );
                }

                process.set_syscall_return_value(rval);
            }
            Syscall::Command {
                driver_number,
                subdriver_number,
                arg0,
                arg1,
            } => {
                let cres = resources
                    .syscall_dispatch()
                    .with_driver(driver_number, |driver| match driver {
                        Some(d) => d.command(subdriver_number, arg0, arg1, process.processid()),
                        None => CommandReturn::failure(ErrorCode::NODEVICE),
                    });

                let res = SyscallReturn::from_command_return(cres);

                if config::CONFIG.trace_syscalls {
                    debug!(
                        "[{:?}] cmd({:#x}, {}, {:#x}, {:#x}) = {:?}",
                        process.processid(),
                        driver_number,
                        subdriver_number,
                        arg0,
                        arg1,
                        res,
                    );
                }
                process.set_syscall_return_value(res);
            }
            Syscall::ReadWriteAllow {
                driver_number,
                subdriver_number,
                allow_address,
                allow_size,
            } => {
                let res = resources
                    .syscall_dispatch()
                    .with_driver(driver_number, |driver| match driver {
                        Some(d) => {
                            // Try to create an appropriate [`ReadWriteProcessBuffer`].
                            // This method will ensure that the memory in question
                            // is located in the process-accessible memory space.
                            match process.build_readwrite_process_buffer(allow_address, allow_size)
                            {
                                Ok(rw_pbuf) => {
                                    // Creating the [`ReadWriteProcessBuffer`] worked,
                                    // provide it to the capsule.
                                    match d.allow_readwrite(
                                        process.processid(),
                                        subdriver_number,
                                        rw_pbuf,
                                    ) {
                                        Ok(returned_pbuf) => {
                                            // The capsule has accepted the allow
                                            // operation. Pass the previous buffer
                                            // information back to the process.
                                            let (ptr, len) = returned_pbuf.consume();
                                            SyscallReturn::AllowReadWriteSuccess(ptr, len)
                                        }
                                        Err((rejected_pbuf, err)) => {
                                            // The capsule has rejected the allow
                                            // operation. Pass the new buffer information
                                            // back to the process.
                                            let (ptr, len) = rejected_pbuf.consume();
                                            SyscallReturn::AllowReadWriteFailure(err, ptr, len)
                                        }
                                    }
                                }
                                Err(allow_error) => {
                                    // There was an error creating the [`ReadWriteProcessBuffer`].
                                    // Report back to the process.
                                    SyscallReturn::AllowReadWriteFailure(
                                        allow_error,
                                        allow_address,
                                        allow_size,
                                    )
                                }
                            }
                        }

                        None => SyscallReturn::AllowReadWriteFailure(
                            ErrorCode::NODEVICE,
                            allow_address,
                            allow_size,
                        ),
                    });

                if config::CONFIG.trace_syscalls {
                    debug!(
                        "[{:?}] read-write allow({:#x}, {}, @{:#x}, {:#x}) = {:?}",
                        process.processid(),
                        driver_number,
                        subdriver_number,
                        allow_address as usize,
                        allow_size,
                        res
                    );
                }
                process.set_syscall_return_value(res);
            }
            Syscall::ReadOnlyAllow {
                driver_number,
                subdriver_number,
                allow_address,
                allow_size,
            } => {
                let res = resources
                    .syscall_dispatch()
                    .with_driver(driver_number, |driver| match driver {
                        Some(d) => {
                            // Try to create an appropriate [`ReadOnlyProcessBuffer`].
                            // This method will ensure that the memory in question
                            // is located in the process-accessible memory space.
                            match process.build_readonly_process_buffer(allow_address, allow_size) {
                                Ok(ro_pbuf) => {
                                    // Creating the [`ReadOnlyProcessBuffer`] worked,
                                    // provide it to the capsule.
                                    match d.allow_readonly(
                                        process.processid(),
                                        subdriver_number,
                                        ro_pbuf,
                                    ) {
                                        Ok(returned_pbuf) => {
                                            // The capsule has accepted the allow
                                            // operation. Pass the previous buffer
                                            // information back to the process.
                                            let (ptr, len) = returned_pbuf.consume();
                                            SyscallReturn::AllowReadOnlySuccess(ptr, len)
                                        }
                                        Err((rejected_pbuf, err)) => {
                                            // The capsule has rejected the allow
                                            // operation. Pass the new buffer information
                                            // back to the process.
                                            let (ptr, len) = rejected_pbuf.consume();
                                            SyscallReturn::AllowReadOnlyFailure(err, ptr, len)
                                        }
                                    }
                                }
                                Err(allow_error) => {
                                    // There was an error creating the
                                    // [`ReadOnlyProcessBuffer`]. Report
                                    // back to the process.
                                    SyscallReturn::AllowReadOnlyFailure(
                                        allow_error,
                                        allow_address,
                                        allow_size,
                                    )
                                }
                            }
                        }
                        None => SyscallReturn::AllowReadOnlyFailure(
                            ErrorCode::NODEVICE,
                            allow_address,
                            allow_size,
                        ),
                    });

                if config::CONFIG.trace_syscalls {
                    debug!(
                        "[{:?}] read-only allow({:#x}, {}, @{:#x}, {:#x}) = {:?}",
                        process.processid(),
                        driver_number,
                        subdriver_number,
                        allow_address as usize,
                        allow_size,
                        res
                    );
                }

                process.set_syscall_return_value(res);
            }
            Syscall::Exit {
                which,
                completion_code,
            } => match which {
                // The process called the `exit-terminate` system call.
                0 => process.terminate(completion_code as u32),
                // The process called the `exit-restart` system call.
                1 => process.try_restart(completion_code as u32),
                // The process called an invalid variant of the Exit
                // system call class.
                _ => process.set_syscall_return_value(SyscallReturn::Failure(ErrorCode::NOSUPPORT)),
            },
        }
    }
}
