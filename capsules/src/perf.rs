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
    syscall::{CommandReturn, SyscallDriver},
    ErrorCode, ProcessId,
};

pub struct Perf;

impl Perf {
    pub fn new() -> Self {
        dwt::enable_cycle_counter();
        Perf {}
    }
}

impl SyscallDriver for Perf {
    /// Control the Perf system.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Get current cycle count.
    fn command(&self, command_num: usize, _data: usize, _: usize, _: ProcessId) -> CommandReturn {
        match command_num {
            0 /* check if present */ => CommandReturn::from(dwt::is_cycle_counter_present()),

            1  =>
                CommandReturn::success_u32( dwt::cycle_count() ),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _processid: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
