// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024.

//! Linker-defined symbols

use kernel::memory_management::pages::Page4KiB;
use kernel::memory_management::slices::MutablePhysicalSlice;
use kernel::utilities::ordering::SmallerPair;
use kernel::utilities::pointers::MutablePointer;
use kernel::utilities::slices::NonEmptyMutableSlice;

extern "C" {
    /// Beginning of the kernel's text segment.
    pub(crate) static mut _stext: u8;
    /// End of the kernel's text segment.
    pub(crate) static mut _etext: u8;
    /// Beginning of the ROM region, i.e. kernel's code and read-only data.
    pub(crate) static mut _srom: u8;
    /// End of the ROM region.
    pub(crate) static mut _erom: u8;
    /// Start of the PROG region, i.e. applications code and read-only data.
    pub(crate) static mut _sprog: u8;
    /// End of the PROG region.
    pub(crate) static mut _eprog: u8;
    /// Begginning of the RAM region.
    pub(crate) static mut _sram: u8;
    /// End of the RAM region.
    pub(crate) static mut _eram: u8;
    /// Beginning of the ROM region containing app images.
    pub(crate) static mut _sapps: u8;
    /// End of the ROM region containing app images.
    pub(crate) static mut _eapps: u8;
    /// Beginning of the RAM region for app memory.
    pub(crate) static mut _sappmem: u8;
    /// End of the RAM region for app memory.
    pub(crate) static mut _eappmem: u8;
    /// Beginning of the peripheral region
    pub(crate) static mut _speripheral: u8;
    /// End of the peripheral region
    pub(crate) static mut _eperipheral: u8;
    /// Beginning of the virtual PROG region containing app images.
    pub(crate) static mut _svirtual_prog: u8;
    /// End of the virtual PROG region containing app images.
    pub(crate) static mut _evirtual_prog: u8;
    /// Beginning of the virtual RAM region.
    pub(crate) static mut _svirtual_ram: u8;
    /// End of the virtual RAM region
    pub(crate) static mut _evirtual_ram: u8;
}

/// # Safety
///
/// 1. Function must be called once per (start, end) pair to ensure no double references are
///    created.
/// 2. The pointers must point to static memory.
/// 3. The pointers must point to physical memory
///
/// # Panic
///
/// 1. `start` and `end` are not page-aligned.
/// 2. address(`start`) >= address(`end`)
unsafe fn get_mutable_physical_slice(
    start: *mut u8,
    end: *mut u8,
) -> MutablePhysicalSlice<'static, Page4KiB> {
    // PANIC: the function's precondition ensures that `start` is page-aligned.
    let start = MutablePointer::new(start.cast()).unwrap();
    // PANIC: the function's precondition ensures that `end` is page-aligned.
    let end = MutablePointer::new(end.cast()).unwrap();
    // PANIC: the function's precondition ensures that `start` < `end`.
    let pointers = SmallerPair::new(start, end).unwrap();
    // SAFETY: the function's precondition ensures that `start` and `end` point to static memory.
    let non_empty_slice = unsafe { NonEmptyMutableSlice::new_start_end(pointers) };

    // SAFETY: the caller ensures that the pointers point to physical memory.
    unsafe { MutablePhysicalSlice::new(non_empty_slice) }
}

/// # Safety
///
/// 1. The function must be called once only to ensure no double mutable references to the kernel's
///    ROM are created.
pub(crate) unsafe fn get_kernel_rom_region() -> MutablePhysicalSlice<'static, Page4KiB> {
    let kernel_rom_start = core::ptr::addr_of_mut!(_srom);
    let kernel_rom_end = core::ptr::addr_of_mut!(_erom);

    // SAFETY:
    //
    // 1. the function's precondition ensures that `get_kernel_rom_region()` is called only once
    //    and as such `get_mutable_physical_slice()` too.
    // 2. both `_srom` and `_erom` are defined as physical pointers.
    // PANIC: the linker script ensures that `kernel_rom_start` and `kernel_rom_end` are
    // page-aligned and have different addresses.
    unsafe { get_mutable_physical_slice(kernel_rom_start, kernel_rom_end) }
}

/// # Safety
///
/// The function must be called once only to ensure no double mutable references to the kernel's
/// PROG are created.
pub(crate) unsafe fn get_kernel_prog_region() -> MutablePhysicalSlice<'static, Page4KiB> {
    let kernel_prog_start = core::ptr::addr_of_mut!(_sprog);
    let kernel_prog_end = core::ptr::addr_of_mut!(_eprog);

    // SAFETY:
    //
    // 1. the function's precondition ensures that `get_kernel_prog_region()` is called only once
    //    and as such `get_mutable_physical_slice()` too.
    // 2. both `_sprog` and `_eprog` are defined as physical pointers.
    // PANIC: the linker script ensures that `kernel_prog_start` and `kernel_prog_end` are
    // page-aligned and have different addresses.
    unsafe { get_mutable_physical_slice(kernel_prog_start, kernel_prog_end) }
}

/// # Safety
///
/// The function must be called once only to ensure no double mutable references to the kernel's
/// RAM are created.
pub(crate) unsafe fn get_kernel_ram_region() -> MutablePhysicalSlice<'static, Page4KiB> {
    let kernel_ram_start = core::ptr::addr_of_mut!(_sram);
    let kernel_ram_end = core::ptr::addr_of_mut!(_eram);

    // SAFETY:
    //
    // 1. the function's precondition ensures that `get_kernel_ram_region()` is called only once
    //    and as such `get_mutable_physical_slice()` too.
    // 2. both `_sram` and `_eram` are defined as physical pointers.
    // PANIC: the linker script ensures that `kernel_ram_start` and `kernel_ram_end` are
    // page-aligned and have different addresses.
    unsafe { get_mutable_physical_slice(kernel_ram_start, kernel_ram_end) }
}

pub(crate) unsafe fn get_kernel_peripheral_region() -> MutablePhysicalSlice<'static, Page4KiB> {
    let kernel_peripheral_start = core::ptr::addr_of_mut!(_speripheral);
    let kernel_peripheral_end = core::ptr::addr_of_mut!(_eperipheral);

    // SAFETY:
    //
    // 1. the function's precondition ensures that `get_kernel_peripheral_region()` is called only
    //    once and as such `get_mutable_physical_slice()` too.
    // 2. both `_speripheral` and `_eperipheral` are defined as physical pointers.
    // PANIC: the linker script ensures that `kernel_peripheral_start` and `kernel_peripheral_end`
    // are page-aligned and have different addresses.
    unsafe { get_mutable_physical_slice(kernel_peripheral_start, kernel_peripheral_end) }
}
