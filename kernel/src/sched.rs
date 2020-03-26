//! Tock core scheduler.

use core::cell::Cell;
use core::ptr::NonNull;

use crate::callback::{AppId, Callback, CallbackId};
use crate::capabilities;
use crate::common::cells::NumericCellExt;
use crate::common::dynamic_deferred_call::DynamicDeferredCall;
use crate::config;
use crate::debug;
use crate::grant::Grant;
use crate::ipc;
use crate::memop;
use crate::platform::mpu::MPU;
use crate::platform::systick::SysTick;
use crate::platform::{Chip, Platform};
use crate::process::{self, Task};
use crate::returncode::ReturnCode;
use crate::syscall::{ContextSwitchReason, Syscall};

/// The time a process is permitted to run before being pre-empted
const KERNEL_TICK_DURATION_US: u32 = 10000;
/// Skip re-scheduling a process if its quanta is nearly exhausted
const MIN_QUANTA_THRESHOLD_US: u32 = 500;

/// Main object for the kernel. Each board will need to create one.
pub struct Kernel {
    /// How many "to-do" items exist at any given time. These include
    /// outstanding callbacks and processes in the Running state.
    work: Cell<usize>,

    /// This holds a pointer to the static array of Process pointers.
    processes: &'static [Option<&'static dyn process::ProcessType>],

    /// A counter which keeps track of how many process identifiers have been
    /// created. This is used to create new unique identifiers for processes.
    process_identifier_max: Cell<usize>,

    /// How many grant regions have been setup. This is incremented on every
    /// call to `create_grant()`. We need to explicitly track this so that when
    /// processes are created they can allocated pointers for each grant.
    grant_counter: Cell<usize>,

    /// Flag to mark that grants have been finalized. This means that the kernel
    /// cannot support creating new grants because processes have already been
    /// created and the data structures for grants have already been
    /// established.
    grants_finalized: Cell<bool>,
}

impl Kernel {
    pub fn new(processes: &'static [Option<&'static dyn process::ProcessType>]) -> Kernel {
        Kernel {
            work: Cell::new(0),
            processes: processes,
            process_identifier_max: Cell::new(0),
            grant_counter: Cell::new(0),
            grants_finalized: Cell::new(false),
        }
    }

    /// Something was scheduled for a process, so there is more work to do.
    crate fn increment_work(&self) {
        self.work.increment();
    }

    /// Something finished for a process, so we decrement how much work there is
    /// to do.
    crate fn decrement_work(&self) {
        self.work.decrement();
    }

    /// Helper function for determining if we should service processes or go to
    /// sleep.
    fn processes_blocked(&self) -> bool {
        self.work.get() == 0
    }

    /// Run a closure on a specific process if it exists. If the process with a
    /// matching `AppId` does not exist at the index specified within the
    /// `AppId`, then `default` will be returned.
    ///
    /// A match will not be found if the process was removed (and there is a
    /// `None` in the process array), if the process changed its identifier
    /// (likely after being restarted), or if the process was moved to a
    /// different index in the processes array. Note that a match _will_ be
    /// found if the process still exists in the correct location in the array
    /// but is in any "stopped" state.
    crate fn process_map_or<F, R>(&self, default: R, appid: AppId, closure: F) -> R
    where
        F: FnOnce(&dyn process::ProcessType) -> R,
    {
        // We use the index in the `appid` so we can do a direct lookup.
        // However, we are not guaranteed that the app still exists at that
        // index in the processes array. To avoid additional overhead, we do the
        // lookup and check here, rather than calling `.index()`.
        let tentative_index = appid.index;

        // Get the process at that index, and if it matches, run the closure
        // on it.
        self.processes
            .get(tentative_index)
            .map_or(None, |process_entry| {
                // Check if there is any process state here, or if the entry is
                // `None`.
                process_entry.map_or(None, |process| {
                    // Check that the process stored here matches the identifier
                    // in the `appid`.
                    if process.appid() == appid {
                        Some(closure(process))
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(default)
    }

    /// Run a closure on every valid process. This will iterate the array of
    /// processes and call the closure on every process that exists.
    crate fn process_each<F>(&self, closure: F)
    where
        F: Fn(&dyn process::ProcessType),
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
    crate fn get_process_iter(
        &self,
    ) -> core::iter::FilterMap<
        core::slice::Iter<Option<&dyn process::ProcessType>>,
        fn(&Option<&'static dyn process::ProcessType>) -> Option<&'static dyn process::ProcessType>,
    > {
        fn keep_some(
            &x: &Option<&'static dyn process::ProcessType>,
        ) -> Option<&'static dyn process::ProcessType> {
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
        F: Fn(&dyn process::ProcessType),
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

    /// Run a closure on every process, but only continue if the closure returns
    /// `FAIL`. That is, if the closure returns any other return code than
    /// `FAIL`, that value will be returned from this function and the iteration
    /// of the array of processes will stop.
    crate fn process_until<F>(&self, closure: F) -> ReturnCode
    where
        F: Fn(&dyn process::ProcessType) -> ReturnCode,
    {
        for process in self.processes.iter() {
            match process {
                Some(p) => {
                    let ret = closure(*p);
                    if ret != ReturnCode::FAIL {
                        return ret;
                    }
                }
                None => {}
            }
        }
        ReturnCode::FAIL
    }

    /// Retrieve the `AppId` of the given app based on its identifier. This is
    /// useful if an app identifier is passed to the kernel from somewhere (such
    /// as from userspace) and needs to be expanded to a full `AppId` for use
    /// with other APIs.
    crate fn lookup_app_by_identifier(&self, identifier: usize) -> Option<AppId> {
        self.processes.iter().find_map(|&p| {
            p.map_or(None, |p2| {
                if p2.appid().id() == identifier {
                    Some(p2.appid())
                } else {
                    None
                }
            })
        })
    }

    /// Checks if the provided `AppId` is still valid given the processes stored
    /// in the processes array. Returns `true` if the AppId still refers to
    /// a valid process, and `false` if not.
    ///
    /// This is needed for `AppId` itself to implement the `.index()` command to
    /// verify that the referenced app is still at the correct index.
    crate fn appid_is_valid(&self, appid: &AppId) -> bool {
        self.processes.get(appid.index).map_or(false, |p| {
            p.map_or(false, |process| process.appid().id() == appid.id())
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
    pub fn create_grant<T: Default>(
        &'static self,
        _capability: &dyn capabilities::MemoryAllocationCapability,
    ) -> Grant<T> {
        if self.grants_finalized.get() {
            panic!("Grants finalized. Cannot create a new grant.");
        }

        // Create and return a new grant.
        let grant_index = self.grant_counter.get();
        self.grant_counter.increment();
        Grant::new(self, grant_index)
    }

    /// Returns the number of grants that have been setup in the system and
    /// marks the grants as "finalized". This means that no more grants can
    /// be created because data structures have been setup based on the number
    /// of grants when this function is called.
    ///
    /// In practice, this is called when processes are created, and the process
    /// memory is setup based on the number of current grants.
    crate fn get_grant_count_and_finalize(&self) -> usize {
        self.grants_finalized.set(true);
        self.grant_counter.get()
    }

    /// Create a new unique identifier for a process and return the identifier.
    ///
    /// Typically we just choose a larger number than we have used for any process
    /// before which ensures that the identifier is unique.
    crate fn create_process_identifier(&self) -> usize {
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

    /// Main loop.
    pub fn kernel_loop<P: Platform, C: Chip>(
        &'static self,
        platform: &P,
        chip: &C,
        ipc: Option<&ipc::IPC>,
        _capability: &dyn capabilities::MainLoopCapability,
    ) {
        loop {
            unsafe {
                chip.service_pending_interrupts();
                DynamicDeferredCall::call_global_instance_while(|| !chip.has_pending_interrupts());

                for p in self.processes.iter() {
                    p.map(|process| {
                        self.do_process(platform, chip, process, ipc);
                    });
                    if chip.has_pending_interrupts()
                        || DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
                    {
                        break;
                    }
                }

                chip.atomic(|| {
                    if !chip.has_pending_interrupts()
                        && !DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
                        && self.processes_blocked()
                    {
                        chip.sleep();
                    }
                });
            };
        }
    }

    unsafe fn do_process<P: Platform, C: Chip>(
        &self,
        platform: &P,
        chip: &C,
        process: &dyn process::ProcessType,
        ipc: Option<&crate::ipc::IPC>,
    ) {
        let systick = chip.systick();
        systick.reset();
        systick.set_timer(KERNEL_TICK_DURATION_US);
        systick.enable(false);

        loop {
            if chip.has_pending_interrupts()
                || DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
            {
                break;
            }

            if systick.overflowed() || !systick.greater_than(MIN_QUANTA_THRESHOLD_US) {
                if config::CONFIG.debug_processes {
                    process.debug_timeslice_expired();
                }
                break;
            }

            match process.get_state() {
                process::State::Running => {
                    // Running means that this process expects to be running,
                    // so go ahead and set things up and switch to executing
                    // the process.
                    process.setup_mpu();
                    chip.mpu().enable_mpu();
                    systick.enable(true);
                    let context_switch_reason = process.switch_to();
                    systick.enable(false);
                    chip.mpu().disable_mpu();

                    // Now the process has returned back to the kernel. Check
                    // why and handle the process as appropriate.
                    match context_switch_reason {
                        Some(ContextSwitchReason::Fault) => {
                            // Let process deal with it as appropriate.
                            process.set_fault_state();
                        }
                        Some(ContextSwitchReason::SyscallFired { syscall }) => {
                            if config::CONFIG.debug_processes {
                                process.debug_syscall_called(syscall);
                            }

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
                            if syscall != Syscall::YIELD {
                                if let Err(response) = platform.filter_syscall(process, &syscall) {
                                    process.set_syscall_return_value(response.into());
                                    continue;
                                }
                            }

                            // Handle each of the syscalls.
                            match syscall {
                                Syscall::MEMOP { operand, arg0 } => {
                                    let res = memop::memop(process, operand, arg0);
                                    if config::CONFIG.trace_syscalls {
                                        debug!(
                                            "[{:?}] memop({}, {:#x}) = {:#x} = {:?}",
                                            process.appid(),
                                            operand,
                                            arg0,
                                            usize::from(res),
                                            res
                                        );
                                    }
                                    process.set_syscall_return_value(res.into());
                                }
                                Syscall::YIELD => {
                                    if config::CONFIG.trace_syscalls {
                                        debug!("[{:?}] yield", process.appid());
                                    }
                                    process.set_yielded_state();

                                    // There might be already enqueued callbacks
                                    continue;
                                }
                                Syscall::SUBSCRIBE {
                                    driver_number,
                                    subdriver_number,
                                    callback_ptr,
                                    appdata,
                                } => {
                                    let callback_id = CallbackId {
                                        driver_num: driver_number,
                                        subscribe_num: subdriver_number,
                                    };
                                    process.remove_pending_callbacks(callback_id);

                                    let callback = NonNull::new(callback_ptr).map(|ptr| {
                                        Callback::new(
                                            process.appid(),
                                            callback_id,
                                            appdata,
                                            ptr.cast(),
                                        )
                                    });

                                    let res =
                                        platform.with_driver(
                                            driver_number,
                                            |driver| match driver {
                                                Some(d) => d.subscribe(
                                                    subdriver_number,
                                                    callback,
                                                    process.appid(),
                                                ),
                                                None => ReturnCode::ENODEVICE,
                                            },
                                        );
                                    if config::CONFIG.trace_syscalls {
                                        debug!(
                                            "[{:?}] subscribe({:#x}, {}, @{:#x}, {:#x}) = {:#x} = {:?}",
                                            process.appid(),
                                            driver_number,
                                            subdriver_number,
                                            callback_ptr as usize,
                                            appdata,
                                            usize::from(res),
                                            res
                                        );
                                    }
                                    process.set_syscall_return_value(res.into());
                                }
                                Syscall::COMMAND {
                                    driver_number,
                                    subdriver_number,
                                    arg0,
                                    arg1,
                                } => {
                                    let res =
                                        platform.with_driver(
                                            driver_number,
                                            |driver| match driver {
                                                Some(d) => d.command(
                                                    subdriver_number,
                                                    arg0,
                                                    arg1,
                                                    process.appid(),
                                                ),
                                                None => ReturnCode::ENODEVICE,
                                            },
                                        );
                                    if config::CONFIG.trace_syscalls {
                                        debug!(
                                            "[{:?}] cmd({:#x}, {}, {:#x}, {:#x}) = {:#x} = {:?}",
                                            process.appid(),
                                            driver_number,
                                            subdriver_number,
                                            arg0,
                                            arg1,
                                            usize::from(res),
                                            res
                                        );
                                    }
                                    process.set_syscall_return_value(res.into());
                                }
                                Syscall::ALLOW {
                                    driver_number,
                                    subdriver_number,
                                    allow_address,
                                    allow_size,
                                } => {
                                    let res = platform.with_driver(driver_number, |driver| {
                                        match driver {
                                            Some(d) => {
                                                match process.allow(allow_address, allow_size) {
                                                    Ok(oslice) => d.allow(
                                                        process.appid(),
                                                        subdriver_number,
                                                        oslice,
                                                    ),
                                                    Err(err) => err, /* memory not valid */
                                                }
                                            }
                                            None => ReturnCode::ENODEVICE,
                                        }
                                    });
                                    if config::CONFIG.trace_syscalls {
                                        debug!(
                                            "[{:?}] allow({:#x}, {}, @{:#x}, {:#x}) = {:#x} = {:?}",
                                            process.appid(),
                                            driver_number,
                                            subdriver_number,
                                            allow_address as usize,
                                            allow_size,
                                            usize::from(res),
                                            res
                                        );
                                    }
                                    process.set_syscall_return_value(res.into());
                                }
                            }
                        }
                        Some(ContextSwitchReason::TimesliceExpired) => {
                            // break to handle other processes.
                            break;
                        }
                        Some(ContextSwitchReason::Interrupted) => {
                            // break to handle other processes.
                            break;
                        }
                        None => {
                            // Something went wrong when switching to this
                            // process. Indicate this by putting it in a fault
                            // state.
                            process.set_fault_state();
                        }
                    }
                }
                process::State::Yielded | process::State::Unstarted => match process.dequeue_task()
                {
                    // If the process is yielded it might be waiting for a
                    // callback. If there is a task scheduled for this process
                    // go ahead and set the process to execute it.
                    None => break,
                    Some(cb) => match cb {
                        Task::FunctionCall(ccb) => {
                            if config::CONFIG.trace_syscalls {
                                debug!(
                                    "[{:?}] function_call @{:#x}({:#x}, {:#x}, {:#x}, {:#x})",
                                    process.appid(),
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
                                    ipc.schedule_callback(process.appid(), otherapp, ipc_type);
                                },
                            );
                        }
                    },
                },
                process::State::Fault => {
                    // We should never be scheduling a process in fault.
                    panic!("Attempted to schedule a faulty process");
                }
                process::State::StoppedRunning => {
                    break;
                    // Do nothing
                }
                process::State::StoppedYielded => {
                    break;
                    // Do nothing
                }
                process::State::StoppedFaulted => {
                    break;
                    // Do nothing
                }
            }
        }
        systick.reset();
    }
}
