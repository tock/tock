// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! `pflash` (parallel NOR / CFI flash) storage for the qemu-rv32-virt chip.
//!
//! QEMU's `virt` machine unconditionally reserves two 32 MiB `pflash`
//! windows at `0x2000_0000` and `0x2200_0000`, regardless of whether a
//! `-drive if=pflash` is actually attached; see
//! `boards/qemu_rv32_virt/src/lib.rs`'s `PFlashRegion` for the corresponding
//! ePMP grant. This chip only exposes the first of those windows (`unit 0`),
//! which is 32 MiB total, organized into 256 KiB erase sectors (128 of
//! them), matching what QEMU configures for these devices
//! (`sector-length = 0x40000`).

use kernel::utilities::registers::ReadWrite;
use qemu_virt_chip::pflash::Pflash as GenericPflash;

/// Base address of `pflash` unit 0.
pub const PFLASH_BASE: usize = 0x2000_0000;

/// Total size, in bytes, of `pflash` unit 0.
pub const PFLASH_SIZE: usize = 32 * 1024 * 1024;

/// [`PFLASH_SIZE`] in 32-bit words.
pub const PFLASH_WORDS: usize = PFLASH_SIZE / 4;

/// Size, in bytes, of a single erase sector.
pub const PFLASH_SECTOR_SIZE: usize = 256 * 1024;

/// [`PFLASH_SECTOR_SIZE`] in 32-bit words.
pub const PFLASH_SECTOR_WORDS: usize = PFLASH_SECTOR_SIZE / 4;

/// MMIO representation of this chip's `pflash` region, as a flat array of
/// 32-bit-wide words.
pub type PflashRegisters = [ReadWrite<u32>; PFLASH_WORDS];

/// The [`qemu_virt_chip::pflash::Pflash`] driver, specialized for this
/// chip's `pflash` geometry: a `PFLASH_WORDS`-sized device organized into
/// `PFLASH_SECTOR_WORDS`-sized pages.
pub type Pflash<'a> = GenericPflash<'a, PFLASH_WORDS, PFLASH_SECTOR_WORDS, PflashPage>;

/// A single erase-sector-sized page of flash, as required by
/// [`kernel::hil::flash::Flash`].
///
/// An example instantiation looks like:
///
/// ```rust, ignore
/// # use kernel::static_init;
/// # use qemu_rv32_virt_chip::pflash::PflashPage;
/// let pagebuffer = unsafe { static_init!(PflashPage, PflashPage::default()) };
/// ```
pub struct PflashPage(pub [u8; PFLASH_SECTOR_SIZE]);

impl Default for PflashPage {
    // A page here is sector-sized (256 KiB) to match this device's erase
    // granularity, unlike most other `Flash` implementations' much smaller
    // pages. This is only ever constructed once into `static` storage via
    // `static_init!`, never actually placed on a live call stack.
    #[allow(clippy::large_stack_arrays, clippy::large_stack_frames)]
    fn default() -> Self {
        Self([0; PFLASH_SECTOR_SIZE])
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
