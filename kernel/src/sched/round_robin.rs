//! Round Robin Scheduler for Tock

use crate::capabilities;
use crate::common::dynamic_deferred_call::DynamicDeferredCall;
use crate::ipc;
use crate::platform::mpu::MPU;
use crate::platform::systick::SysTick;
use crate::platform::{Chip, Platform};
use crate::process;
use crate::sched::{Kernel, Scheduler};
use crate::syscall::{ContextSwitchReason, Syscall};
use core::cell::Cell;

// TODO: Not requiring this state to be Copy should be possible and more efficient

/// Stores per process state when using the round robin scheduler
#[derive(Copy, Clone, Default)]
pub struct RRProcState {
    /// To prevent unfair situations that can be created by one app consistently
    /// scheduling an interrupt, yeilding, and then interrupting the subsequent app
    /// shortly after it begins executing, we track the portion of a timeslice that a process
    /// has used and allow it to continue after being interrupted.
    us_used_this_timeslice: u32,
}

// Currently relies on assumption that x processes will reside in first x slots of process array
pub struct RoundRobinSched {
    kernel: &'static Kernel,
    num_procs_installed: usize,
    next_up: Cell<usize>,
    proc_states: &'static mut [Option<RRProcState>],
}

impl RoundRobinSched {
    /// How long a process can run before being pre-empted
    const DEFAULT_TIMESLICE_US: u32 = 10000;
    /// Skip re-scheduling a process if its quanta is nearly exhausted
    const MIN_QUANTA_THRESHOLD_US: u32 = 500;
    pub fn new(
        kernel: &'static Kernel,
        proc_states: &'static mut [Option<RRProcState>],
    ) -> RoundRobinSched {
        //have to initialize proc state bc default() sets them to None
        let mut num_procs = 0;
        for (i, s) in proc_states.iter_mut().enumerate() {
            if kernel.processes[i].is_some() {
                num_procs += 1;
                *s = Some(Default::default());
            }
        }
        RoundRobinSched {
            kernel: kernel,
            num_procs_installed: num_procs,
            next_up: Cell::new(0),
            proc_states: proc_states,
        }
    }

    unsafe fn do_process<P: Platform, C: Chip>(
        &mut self,
        platform: &P,
        chip: &C,
        process: &dyn process::ProcessType,
        ipc: Option<&crate::ipc::IPC>,
        proc_timeslice_us: u32,
    ) -> (bool, Option<ContextSwitchReason>) {
        let appid = process.appid();
        let systick = chip.systick();
        systick.reset();
        systick.set_timer(proc_timeslice_us);
        systick.enable(false);
        //track that process was given a chance to execute (bc of case where process has a callback
        //waiting, the callback is handled, then interrupt arrives can cause process not to get a
        //chance to run if that callback being handled puts it in the running state)
        let mut given_chance = false;
        let mut switch_reason_opt = None;
        let mut first = true;

        loop {
            // if this is the first time this loop has iterated, dont break in the
            // case of interrupts. This allows for the scheduler to schedule processes
            // even with interrupts pending if it so chooses.
            if !first {
                if chip.has_pending_interrupts() {
                    break;
                }
            } else {
                first = false;
            }

            if systick.overflowed() || !systick.greater_than(Self::MIN_QUANTA_THRESHOLD_US) {
                process.debug_timeslice_expired();
                break;
            }

            match process.get_state() {
                process::State::Running => {
                    // Running means that this process expects to be running,
                    // so go ahead and set things up and switch to executing
                    // the process.
                    given_chance = true;
                    process.setup_mpu();
                    chip.mpu().enable_mpu();
                    systick.enable(true); //Enables systick interrupts
                    let context_switch_reason = process.switch_to();
                    let us_used = proc_timeslice_us - systick.get_value();
                    systick.enable(false); //disables systick interrupts
                    chip.mpu().disable_mpu();
                    self.proc_states[appid.idx()]
                        .as_mut()
                        .map(|mut state| state.us_used_this_timeslice += us_used);
                    switch_reason_opt = context_switch_reason;

                    // Now the process has returned back to the kernel. Check
                    // why and handle the process as appropriate.
                    self.kernel
                        .process_return(appid, context_switch_reason, process, platform);
                    match context_switch_reason {
                        Some(ContextSwitchReason::SyscallFired {
                            syscall: Syscall::YIELD,
                        }) => {
                            // There might be already enqueued callbacks
                            continue;
                        }
                        Some(ContextSwitchReason::TimesliceExpired) => {
                            // break to handle other processes
                            break;
                        }
                        Some(ContextSwitchReason::Interrupted) => {
                            // break to handle other processes
                            break;
                        }
                        _ => {}
                    }
                }
                process::State::Yielded | process::State::Unstarted => match process.dequeue_task()
                {
                    // If the process is yielded it might be waiting for a
                    // callback. If there is a task scheduled for this process
                    // go ahead and set the process to execute it.
                    None => {
                        given_chance = true;
                        break;
                    }
                    Some(cb) => self.kernel.handle_callback(cb, process, ipc),
                },
                process::State::Fault => {
                    // We should never be scheduling a process in fault.
                    panic!("Attempted to schedule a faulty process");
                }
                process::State::StoppedRunning => {
                    given_chance = true;
                    break;
                    // Do nothing
                }
                process::State::StoppedYielded => {
                    given_chance = true;
                    break;
                    // Do nothing
                }
                process::State::StoppedFaulted => {
                    given_chance = true;
                    break;
                    // Do nothing
                }
            }
        }
        systick.reset();
        (given_chance, switch_reason_opt)
    }
}

impl Scheduler for RoundRobinSched {
    type ProcessState = RRProcState;

    /// Main loop.
    fn kernel_loop<P: Platform, C: Chip>(
        &'static mut self,
        platform: &P,
        chip: &C,
        ipc: Option<&ipc::IPC>,
        _capability: &dyn capabilities::MainLoopCapability,
    ) {
        loop {
            unsafe {
                chip.service_pending_interrupts();
                DynamicDeferredCall::call_global_instance_while(|| !chip.has_pending_interrupts());

                loop {
                    let next = self.next_up.get();
                    if chip.has_pending_interrupts()
                        || DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
                        || self.kernel.processes_blocked()
                        || self.num_procs_installed == 0
                    {
                        break;
                    }
                    self.kernel.processes[next].map(|process| {
                        let timeslice_us = Self::DEFAULT_TIMESLICE_US
                            - self.proc_states[next].unwrap().us_used_this_timeslice;
                        let (given_chance, switch_reason) =
                            self.do_process(platform, chip, process, ipc, timeslice_us);

                        if given_chance {
                            let mut reschedule = false;
                            let used_so_far =
                                self.proc_states[next].unwrap().us_used_this_timeslice;
                            if switch_reason == Some(ContextSwitchReason::Interrupted) {
                                if Self::DEFAULT_TIMESLICE_US - used_so_far
                                    >= Self::MIN_QUANTA_THRESHOLD_US
                                {
                                    self.proc_states[next].as_mut().map(|mut state| {
                                        state.us_used_this_timeslice = used_so_far;
                                    });
                                    reschedule = true; //Was interrupted before using entire timeslice!
                                }
                                // want to inform scheduler of time passed and to reschedule
                            }
                            if !reschedule {
                                self.proc_states[next].as_mut().map(|mut state| {
                                    state.us_used_this_timeslice = 0;
                                });
                                if next < self.num_procs_installed - 1 {
                                    self.next_up.set(next + 1);
                                } else {
                                    self.next_up.set(0);
                                }
                            }
                        }
                    });
                }

                chip.atomic(|| {
                    if !chip.has_pending_interrupts()
                        && !DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
                        && self.kernel.processes_blocked()
                    {
                        chip.sleep();
                    }
                });
            };
        }
    }
}
