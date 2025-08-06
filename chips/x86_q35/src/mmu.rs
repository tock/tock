// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Small helpers for 32-bit paging on the Q35 machine.

// First–level page-directory and entry types that the board already uses.
use x86::registers::bits32::paging::PDEntry;
pub use x86::registers::bits32::paging::PD;

/// Physical address where Bochs/QEMU exposes the linear-frame-buffer BAR.
pub const LFB_PHYS_BASE: u32 = 0xE0_00_0000;

/// Bit flags for a 32-bit page-directory entry.
pub mod pde_flags {
    /// Entry present in memory.
    pub const PRESENT: u32 = 1 << 0;
    /// Writable by supervisor.
    pub const WRITABLE: u32 = 1 << 1;
    /// Page size = 4 MiB (sets PS bit).
    pub const PAGE_SIZE_4MIB: u32 = 1 << 7;
}

/// Map the 4 MiB linear-frame-buffer region into the given page directory.
///
/// The mapping is *identity* (virtual == physical) because the board runs
/// in a flat 1 GiB address space.
pub fn map_linear_framebuffer(page_dir: &mut PD) {
    let idx = (LFB_PHYS_BASE >> 22) as usize; // top 10 bits → PDE index
    let entry_value = (LFB_PHYS_BASE & 0xFFC0_0000)
        | pde_flags::PRESENT
        | pde_flags::WRITABLE
        | pde_flags::PAGE_SIZE_4MIB;

    page_dir[idx] = PDEntry(entry_value);
}
