// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementation of the MEMOP family of syscalls.

use crate::process::Process;
use crate::syscall::SyscallReturn;
use crate::utilities::capability_ptr::{CapabilityPtr, CapabilityPtrPermissions};
use crate::ErrorCode;

/// Handle the `memop` syscall.
///
/// ### `memop_num`
///
/// - `0`: BRK. Change the location of the program break and return a
///   SyscallReturn.
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
pub(crate) fn memop(process: &dyn Process, op_type: usize, r1: usize) -> SyscallReturn {
    match op_type {
        // Op Type 0: BRK
        0 => process
            .brk(r1 as *const u8)
            .map(|_| SyscallReturn::Success)
            .unwrap_or(SyscallReturn::Failure(ErrorCode::NOMEM)),

        // Op Type 1: SBRK
        1 => process
            .sbrk(r1 as isize)
            .map(SyscallReturn::SuccessPtr)
            .unwrap_or(SyscallReturn::Failure(ErrorCode::NOMEM)),

        // Op Type 2: Process memory start
        2 => SyscallReturn::SuccessPtr(unsafe {
            let addresses = process.get_addresses();
            CapabilityPtr::new_with_authority(
                addresses.sram_start as *const _,
                addresses.sram_start,
                addresses.sram_app_brk - addresses.sram_start,
                CapabilityPtrPermissions::ReadWrite,
            )
        }),

        // Op Type 3: Process memory end
        3 => SyscallReturn::SuccessPtr(unsafe {
            let addresses = process.get_addresses();
            CapabilityPtr::new_with_authority(
                addresses.sram_end as *const _,
                addresses.sram_start,
                addresses.sram_end - addresses.sram_start,
                CapabilityPtrPermissions::ReadWrite,
            )
        }),

        // Op Type 4: Process flash start
        4 => SyscallReturn::SuccessPtr(unsafe {
            let addresses = process.get_addresses();
            CapabilityPtr::new_with_authority(
                addresses.flash_start as *const _,
                addresses.flash_start,
                addresses.flash_end - addresses.flash_start,
                CapabilityPtrPermissions::Execute,
            )
        }),

        // Op Type 5: Process flash end
        5 => SyscallReturn::SuccessPtr(unsafe {
            let addresses = process.get_addresses();
            CapabilityPtr::new_with_authority(
                addresses.flash_end as *const _,
                addresses.flash_start,
                addresses.flash_end - addresses.flash_start,
                CapabilityPtrPermissions::Execute,
            )
        }),

        // Op Type 6: Grant region begin
        6 => SyscallReturn::SuccessAddr(process.get_addresses().sram_grant_start),

        // Op Type 7: Number of defined writeable regions in the TBF header.
        7 => SyscallReturn::SuccessU32(process.number_writeable_flash_regions() as u32),

        // Op Type 8: The start address of the writeable region indexed by r1.
        8 => {
            let flash_start = process.get_addresses().flash_start;
            let (offset, size) = process.get_writeable_flash_region(r1);
            if size == 0 {
                SyscallReturn::Failure(ErrorCode::FAIL)
            } else {
                SyscallReturn::SuccessPtr(unsafe {
                    CapabilityPtr::new_with_authority(
                        (flash_start + offset) as *const _,
                        flash_start + offset,
                        size,
                        CapabilityPtrPermissions::ReadWrite,
                    )
                })
            }
        }

        // Op Type 9: The end address of the writeable region indexed by r1.
        // Returns (void*) -1 on failure, meaning the selected writeable region
        // does not exist.
        9 => {
            let flash_start = process.get_addresses().flash_start;
            let (offset, size) = process.get_writeable_flash_region(r1);
            if size == 0 {
                SyscallReturn::Failure(ErrorCode::FAIL)
            } else {
                SyscallReturn::SuccessPtr(unsafe {
                    CapabilityPtr::new_with_authority(
                        (flash_start + offset + size) as *const _,
                        flash_start + offset,
                        size,
                        CapabilityPtrPermissions::ReadWrite,
                    )
                })
            }
        }

        // Op Type 10: Specify where the start of the app stack is.
        10 => {
            process.update_stack_start_pointer(r1 as *const u8);
            SyscallReturn::Success
        }

        // Op Type 11: Specify where the start of the app heap is.
        11 => {
            process.update_heap_start_pointer(r1 as *const u8);
            SyscallReturn::Success
        }

        _ => SyscallReturn::Failure(ErrorCode::NOSUPPORT),
    }
}
