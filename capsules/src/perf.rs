//! Provides a performance counter interface for userspace.
//!
//! Usage
//! -----
//!
//! When loading this capsule, the required performance counters are activated.

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Perf as usize;

use cortexm::{dcb, dwt};
use kernel::{
    hil,
    syscall::{CommandReturn, SyscallDriver},
    ErrorCode, ProcessId,
};

pub struct Perf<'a, P: hil::debug::PerformanceCounters> {
    counters: &'a P,
}

impl<'a, P: hil::debug::PerformanceCounters> Perf<'a, P> {
    pub fn new(counters: &'a P) -> Self {
        P::enable_cycle_counter();
        Perf { counters }
    }
}

impl<'a, P: hil::debug::PerformanceCounters> SyscallDriver for Perf<'a, P> {
    /// Control the Perf system.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Get current cycle count.
    fn command(&self, command_num: usize, _data: usize, _: usize, _: ProcessId) -> CommandReturn {
        match command_num {
            0 /* check if present */ => CommandReturn::success(),

            1  =>
                CommandReturn::success_u32(P::cycle_count()),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _processid: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
