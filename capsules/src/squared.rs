//! Provide userspace service to calculate the square of a number.
//!
//! This uses the U32U64 command return type.

use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::ErrorCode;
use kernel::ProcessId;

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Squared as usize;

pub struct Squared {}

impl SyscallDriver for Squared {
    fn command(
        &self,
        command_type: usize,
        arg1: usize,
        _: usize,
        _appid: ProcessId,
    ) -> CommandReturn {
        match command_type {
            0 => CommandReturn::success(),

            // Square a number.
            1 => {
                let multiplicand: u64 = arg1 as u64;
                let multiplier: u64 = arg1 as u64;
                let product = multiplicand * multiplier;

                CommandReturn::success_u32_u64(0, product)
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
    fn allocate_grant(&self, _: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
