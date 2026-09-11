// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! `pflash` (parallel NOR / CFI flash) storage for the qemu-rv32-virt chip.
//!
//! QEMU's `virt` machine unconditionally reserves two 32 MiB pflash windows at
//! `0x2000_0000` and `0x2200_0000`, regardless of whether a `-drive if=pflash`
//! is actually attached.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::ReadWrite;
use qemu_virt_chip::pflash::Pflash;

/// Total size, in bytes, of `pflash` unit 0.
pub const PFLASH0_SIZE: usize = 32 * 1024 * 1024;

/// `pflash` unit 0 size in 32-bit words.
pub const PFLASH0_WORDS: usize = PFLASH0_SIZE / 4;

/// Size, in bytes, of a single erase sector.
pub const PFLASH0_SECTOR_SIZE: usize = 256 * 1024;

/// Sector size in 32-bit words.
pub const PFLASH0_SECTOR_WORDS: usize = PFLASH0_SECTOR_SIZE / 4;

/// MMIO representation of this chip's `pflash` region.
///
/// This is simply a flat array of 32-bit-wide words.
pub type Pflash0Registers = [ReadWrite<u32>; PFLASH0_WORDS];

/// The [`Pflash`] driver, specialized for the qemu-rv32-virt chip.
pub type Pflash0<'a> = Pflash<'a, PFLASH0_WORDS, PFLASH0_SECTOR_WORDS, PflashPage>;

pub const PFLASH0_BASE: StaticRef<Pflash0Registers> =
    unsafe { StaticRef::new(0x2000_0000 as *const Pflash0Registers) };

/// A single erase-sector-sized page of flash.
///
/// An example instantiation looks like:
///
/// ```rust,ignore
/// # use kernel::static_init;
/// # use qemu_rv32_virt_chip::pflash::PflashPage;
/// let pagebuffer = unsafe { static_init!(PflashPage, PflashPage::default()) };
/// ```
pub struct PflashPage(pub [u8; PFLASH0_SECTOR_SIZE]);

impl Default for PflashPage {
    // A page here is sector-sized (256 KiB) to match this device's erase
    // granularity, unlike most other `Flash` implementations' much smaller
    // pages. This is only ever constructed once into `static` storage via
    // `static_init!`, never actually placed on a live call stack.
    #[allow(clippy::large_stack_arrays, clippy::large_stack_frames)]
    fn default() -> Self {
        Self([0; PFLASH0_SECTOR_SIZE])
    }
}

impl core::ops::Index<usize> for PflashPage {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl core::ops::IndexMut<usize> for PflashPage {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for PflashPage {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
