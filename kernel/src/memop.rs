//! Implementation of the MEMOP family of syscalls.

use process::Process;
use returncode::ReturnCode;

/// Handle the `memop` syscall.
///
/// ### `memop_num`
///
/// - `0`: BRK. Change the location of the program break and return a
///   ReturnCode.
/// - `1`: SBRK. Change the location of the program break and return the
///   previous break address.
/// - `2`: Get the address of the start of the application's RAM allocation.
/// - `3`: Get the address pointing to the first address after the end of the
///   application's RAM allocation.
/// - `4`: Get the address of the start of the application's flash region. This
///   is where the TBF header is located.
/// - `5`: Get the address pointing to the first address after the end of the
///   application's flash region.
/// - `6`: Get the address of the lowest address of the grant region for the
///   app.
/// - `7`: Get the number of writeable flash regions defined in the header of
///   this app.
/// - `8`: Get the start address of the writeable region indexed from 0 by r1.
///   Returns (void*) -1 on failure, meaning the selected writeable region
///   does not exist.
/// - `9`: Get the end address of the writeable region indexed by r1. Returns
///   (void*) -1 on failure, meaning the selected writeable region does not
///   exist.
/// - `10`: Specify where the start of the app stack is. This tells the kernel
///   where the app has put the start of its stack. This is not strictly
///   necessary for correct operation, but allows for better debugging if the
///   app crashes.
/// - `11`: Specify where the start of the app heap is. This tells the kernel
///   where the app has put the start of its heap. This is not strictly
///   necessary for correct operation, but allows for better debugging if the
///   app crashes.
pub fn memop(process: &mut Process) -> ReturnCode {
    let op_type = process.r0();
    let r1 = process.r1();

    match op_type {
        // Op Type 0: BRK
        0 /* BRK */ => {
            process.brk(r1 as *const u8)
                .map(|_| ReturnCode::SUCCESS)
                .unwrap_or(ReturnCode::ENOMEM)
        },

        // Op Type 1: SBRK
        1 /* SBRK */ => {
            process.sbrk(r1 as isize)
                .map(|addr| ReturnCode::SuccessWithValue { value: addr as usize })
                .unwrap_or(ReturnCode::ENOMEM)
        },

        // Op Type 2: Process memory start
        2 => ReturnCode::SuccessWithValue { value: process.mem_start() as usize },

        // Op Type 3: Process memory end
        3 => ReturnCode::SuccessWithValue { value: process.mem_end() as usize },

        // Op Type 4: Process flash start
        4 => ReturnCode::SuccessWithValue { value: process.flash_start() as usize },

        // Op Type 5: Process flash end
        5 => ReturnCode::SuccessWithValue { value: process.flash_end() as usize },

        // Op Type 6: Grant region begin
        6 => ReturnCode::SuccessWithValue { value: process.kernel_memory_break() as usize },

        // Op Type 7: Number of defined writeable regions in the TBF header.
        7 => ReturnCode::SuccessWithValue { value: process.number_writeable_flash_regions() },

        // Op Type 8: The start address of the writeable region indexed by r1.
        8 => {
            let flash_start = process.flash_start() as usize;
            let (offset, size) = process.get_writeable_flash_region(r1);
            if size == 0 {
                ReturnCode::FAIL
            } else {
                ReturnCode::SuccessWithValue { value: flash_start + offset as usize }
            }
        }

        // Op Type 9: The end address of the writeable region indexed by r1.
        // Returns (void*) -1 on failure, meaning the selected writeable region
        // does not exist.
        9 => {
            let flash_start = process.flash_start() as usize;
            let (offset, size) = process.get_writeable_flash_region(r1);
            if size == 0 {
                ReturnCode::FAIL
            } else {
                ReturnCode::SuccessWithValue { value: flash_start +
                                                      offset as usize +
                                                      size as usize }
            }
        }

        // Op Type 10: Specify where the start of the app stack is.
        10 => {
            process.update_stack_start_pointer(r1 as *const u8);
            ReturnCode::SUCCESS
        }

        // Op Type 11: Specify where the start of the app heap is.
        11 => {
            process.update_heap_start_pointer(r1 as *const u8);
            ReturnCode::SUCCESS
        }

        _ => ReturnCode::ENOSUPPORT,
    }
}
