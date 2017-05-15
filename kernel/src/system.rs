//! Provide capsule driver for interacting with the Tock kernel.
//! This allows apps to query information about themselves and the running
//! kernel.

use {AppId, Driver};
use process;
use returncode::ReturnCode;

pub struct System {}

impl System {
    pub unsafe fn new() -> System {
        System {}
    }
}

impl Driver for System {
    fn command(&self, command_num: usize, _data: usize, appid: AppId) -> ReturnCode {
        let maybe_process = process::get_process_from_appid(appid);

        match command_num {
            /// Command 0: Major version of Tock Kernel
            0 => ReturnCode::SuccessWithValue { value: 0 },

            /// Command 1: Process memory start
            1 => {
                maybe_process.map_or(ReturnCode::FAIL, |p| {
                    ReturnCode::SuccessWithValue { value: p.mem_start() as usize }
                })
            }

            /// Command 2: Process memory end
            2 => {
                maybe_process.map_or(ReturnCode::FAIL, |p| {
                    ReturnCode::SuccessWithValue { value: p.mem_end() as usize }
                })
            }

            /// Command 3: Process flash start
            3 => {
                maybe_process.map_or(ReturnCode::FAIL, |p| {
                    ReturnCode::SuccessWithValue { value: p.flash_start() as usize }
                })
            }

            /// Command 4: Process flash end
            4 => {
                maybe_process.map_or(ReturnCode::FAIL, |p| {
                    ReturnCode::SuccessWithValue { value: p.flash_end() as usize }
                })
            }

            /// Command 5: Grant region begin
            5 => {
                maybe_process.map_or(ReturnCode::FAIL, |p| {
                    ReturnCode::SuccessWithValue { value: p.kernel_memory_break() as usize }
                })
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
