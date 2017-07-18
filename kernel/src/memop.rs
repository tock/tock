//! Implementation of the MEMOP family of syscalls.

use process::Process;
use returncode::ReturnCode;

pub fn memop(process: &mut Process) -> ReturnCode {
    let op_type = process.r0();
    let r1 = process.r1();

    match op_type {
        /// Op Type 0: BRK
        0 /* BRK */ => {
            process.brk(r1 as *const u8)
                .map(|_| ReturnCode::SUCCESS)
                .unwrap_or(ReturnCode::ENOMEM)
        },

        /// Op Type 1: SBRK
        1 /* SBRK */ => {
            process.sbrk(r1 as isize)
                .map(|addr| ReturnCode::SuccessWithValue { value: addr as usize })
                .unwrap_or(ReturnCode::ENOMEM)
        },

        /// Op Type 2: Process memory start
        2 => ReturnCode::SuccessWithValue { value: process.mem_start() as usize },

        /// Op Type 3: Process memory end
        3 => ReturnCode::SuccessWithValue { value: process.mem_end() as usize },

        /// Op Type 4: Process flash start
        4 => ReturnCode::SuccessWithValue { value: process.flash_start() as usize },

        /// Op Type 5: Process flash end
        5 => ReturnCode::SuccessWithValue { value: process.flash_end() as usize },

        /// Op Type 6: Grant region begin
        6 => ReturnCode::SuccessWithValue { value: process.kernel_memory_break() as usize },

        _ => ReturnCode::ENOSUPPORT,
    }
}
