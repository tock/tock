//! Provides a cycle counter interface for userspace.
//!
//! Usage
//! -----
//!
//! This capsule is intended for debug purposes, and may return innaccurate results
//! if the counter is started or stopped by multiple apps simultaneously.
//! To enable such cross-app debugging, we do not restrict access to a single app.

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::CycleCount as usize;

use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{hil, ErrorCode, ProcessId};

pub struct CycleCount<'a, P: hil::hw_debug::CycleCounter> {
    counters: &'a P,
}

impl<'a, P: hil::hw_debug::CycleCounter> CycleCount<'a, P> {
    pub fn new(counters: &'a P) -> Self {
        Self { counters }
    }
}

impl<'a, P: hil::hw_debug::CycleCounter> SyscallDriver for CycleCount<'a, P> {
    /// Control the CycleCount system.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Start the cycle counter.
    /// - `2`: Get current cycle count.
    /// - `3`: Reset and stop the cycle counter.
    /// - `4`: Stop the cycle counter.
    fn command(&self, command_num: usize, _data: usize, _: usize, _: ProcessId) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),

            1 => {
                self.counters.start();
                CommandReturn::success()
            }
            2 => CommandReturn::success_u32(self.counters.count()),
            3 => {
                self.counters.reset();
                CommandReturn::success()
            }
            4 => {
                self.counters.stop();
                CommandReturn::success()
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _processid: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
