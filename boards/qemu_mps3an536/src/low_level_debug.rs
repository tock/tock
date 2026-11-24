//! Low-level debug syscall driver.

use kernel::errorcode::ErrorCode;
use kernel::process::{Error, ProcessId};
use kernel::syscall::{CommandReturn, SyscallDriver};

// Low-level debug driver just quickly recoded as it was much faster than to
// understand how to instanciate the official one!!!
// It also allowed to add temporary debug features without messing everything.

pub struct MyLowLevelDebug;

impl MyLowLevelDebug {
    pub const fn new() -> Self {
        Self
    }
}

impl SyscallDriver for MyLowLevelDebug {
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => semihosting::print!(
                "\x1b[0;34m   LowLevelDebug: Dropped from PID {processid:?}!\n\x1b[0m"
            ),
            1 => semihosting::print!(
                "\x1b[0;34m   LowLevelDebug: Alert {} from PID {processid:?}!\n\x1b[0m",
                match data1 {
                    1 => "PANIC",
                    2 => "WRONG LOCATION",
                    _ => "UNKWON",
                }
            ),
            2 => semihosting::print!(
                "\x1b[0;34m   LowLevelDebug: Print1 <{data1}> from PID {processid:?}!\n\x1b[0m"
            ),
            3 => semihosting::print!(
                "\x1b[0;34m   LowLevelDebug: Print2 <{data1}> <{data2}> from PID {processid:?}!\n\x1b[0m"
            ),
            _ => return CommandReturn::failure(ErrorCode::NOSUPPORT),
        }

        CommandReturn::success_u32_u32_u32(512, 513, 514)
    }

    fn allocate_grant(&self, _processid: ProcessId) -> Result<(), Error> {
        // whatever
        Ok(())
    }
}
