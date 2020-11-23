//! Tock 2.0 system call test capsule

use kernel::{AppId, CommandResult, Driver, ErrorCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Tock2Test as usize;

pub struct Tock2TestDriver;

impl Tock2TestDriver {
    pub fn new() -> Tock2TestDriver {
        Tock2TestDriver
    }
}

impl Driver for Tock2TestDriver {
    /// Dummy command return variant test
    fn command(&self, cmd_num: usize, _arg1: usize, _arg2: usize, _appid: AppId) -> CommandResult {
        match cmd_num {
            0 /* check if present */ => CommandResult::success(),
            1 => CommandResult::failure(ErrorCode::FAIL),
	    2 => CommandResult::failure_u32(ErrorCode::BUSY, 0x12345678),
	    3 => CommandResult::failure_u32_u32(ErrorCode::BUSY, 0x87654321, 0xABBACDDC),
	    4 => CommandResult::failure_u64(ErrorCode::BUSY, 0x0F1E2D3C4B5A6978),
	    5 => CommandResult::success_u32(0xDEADBEEF),
	    6 => CommandResult::success_u32_u32(0xBAADF00D, 0xDEADC0DE),
	    7 => CommandResult::success_u32_u32_u32(0xC0DEBA5E, 0xCAFEBABE, 0xDEADFACE),
	    8 => CommandResult::success_u64(0x89ABFEDC01237654),
	    9 => CommandResult::success_u64_u32(0x02468ACE13579BDF, 0xC001D00D),
            _ => CommandResult::failure(ErrorCode::NOSUPPORT),
        }
    }
}
