//! Multilevel feedback queue scheduler for Tock
//! This is a rather unusual implementation where priority
//! levels are just used to assign varying timeslices but
//! do not affect the (round robin) order in which processes execute.
//! This scheduler is mostly intended as an example that varied schedulers
//! are possible with the new scheduler trait, but will likely need work
//! to become a scheduler that actually makes sense to use.

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

#[derive(Copy, Clone, Default)]
pub struct MfProcState {
    /// To prevent unfair situations that can be created by one app consistently
    /// scheduling an interrupt, yeilding, and then interrupting the subsequent app
    /// shortly after it begins executing, we track the portion of a timeslice that a process
    /// has used and allow it to continue after being interrupted.
    us_used_this_timeslice: u32,

    /// Number of times this app has executed
    times_executed: u32,

    /// Average number of us used by this process when it executes
    avg_us_used: u32,

    /// Priority of this process
    priority: u32,

    /// Current timeslice assigned
    cur_timeslice: Option<u32>,
}

/// Implementation of a scheduler that punishes processes which frequently
/// exceed their timeslice or which use more CPU time on average than other
/// processes on the board by assigning these processes smaller timeslices,
/// and rewarding processes which behave better than others with larger timeslices,
/// within a set range of possible timeslices.
pub struct MultiFeedbackSched {
    kernel: &'static Kernel,
    num_procs_installed: usize,
    next_up: Cell<usize>,
    proc_states: &'static mut [Option<MfProcState>],
}

impl MfProcState {
    fn update_avg_exec_time(&mut self, us: u32) {
        let mut_us = if us > 0 { us } else { 1 };
        let prev = self.times_executed as u64;
        self.times_executed += 1;
        let mov_avg = ((self.avg_us_used as u64) * prev + mut_us as u64) / (prev + 1);
        self.avg_us_used = mov_avg as u32;
    }
}

impl MultiFeedbackSched {
    const MAX_TIMESLICE_US: u32 = 20000;
    const MIN_TIMESLICE_US: u32 = 5000;
    const MIN_QUANTA_THRESHOLD_US: u32 = 500;
    pub fn new(
        kernel: &'static Kernel,
        proc_states: &'static mut [Option<MfProcState>],
    ) -> MultiFeedbackSched {
        let mut num_procs = 0;
        for (i, s) in proc_states.iter_mut().enumerate() {
            if kernel.processes[i].is_some() {
                num_procs += 1;
                *s = Some(Default::default());
            }
        }
        MultiFeedbackSched {
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
                switch_reason_opt = Some(ContextSwitchReason::TimesliceExpired);
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

impl Scheduler for MultiFeedbackSched {
    type ProcessState = MfProcState;

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
                    if chip.has_pending_interrupts()
                        || DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
                        || self.kernel.processes_blocked()
                        || self.num_procs_installed == 0
                    {
                        break;
                    }
                    let next = self.next_up.get();
                    self.kernel.processes[next].map(|process| {
                        let state = self.proc_states[next].unwrap();
                        let exec_time = state.cur_timeslice.unwrap_or_else(|| {
                            // Need to assign new timeslice based on priority
                            let (priority, avg_time) = (state.priority, state.avg_us_used);
                            let mut num_higher_priority = 0;
                            for p in self.proc_states.as_ref() {
                                p.map(|proc| {
                                    if proc.priority == priority {
                                        if proc.avg_us_used < avg_time {
                                            num_higher_priority += 1;
                                        }
                                    } else if proc.priority < priority {
                                        num_higher_priority += 1;
                                    }
                                });
                            }
                            let spacing = (Self::MAX_TIMESLICE_US - Self::MIN_TIMESLICE_US)
                                / self.num_procs_installed as u32;
                            let cur_slice =
                                Self::MAX_TIMESLICE_US - (spacing * num_higher_priority);
                            self.proc_states[next].as_mut().map(|mut s| {
                                s.cur_timeslice = Some(cur_slice);
                            });
                            cur_slice
                        });
                        let (given_chance, switch_reason) =
                            self.do_process(platform, chip, process, ipc, exec_time);

                        if given_chance {
                            let mut reschedule = false;
                            let mut used_so_far =
                                self.proc_states[next].unwrap().us_used_this_timeslice;
                            if switch_reason == Some(ContextSwitchReason::TimesliceExpired) {
                                self.proc_states[next].as_mut().map(|mut state| {
                                    state.priority += 1;
                                });
                                used_so_far =
                                    self.proc_states[next].unwrap().cur_timeslice.unwrap();
                            } else if switch_reason == Some(ContextSwitchReason::Interrupted) {
                                if self.proc_states[next].unwrap().cur_timeslice.unwrap()
                                    - used_so_far
                                    >= Self::MIN_QUANTA_THRESHOLD_US
                                {
                                    self.proc_states[next].map(|mut state| {
                                        state.us_used_this_timeslice = used_so_far;
                                    });
                                    reschedule = true; //Was interrupted before using entire timeslice!
                                }
                            } else {
                            }
                            if !reschedule {
                                self.proc_states[next].as_mut().map(|mut state| {
                                    state.update_avg_exec_time(used_so_far);
                                    state.us_used_this_timeslice = 0;
                                    state.cur_timeslice = None;
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
