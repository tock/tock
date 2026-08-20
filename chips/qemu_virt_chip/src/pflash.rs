// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Driver for QEMU's emulated parallel NOR (CFI) flash, as exposed on the
//! `riscv virt` machine's `pflash` devices (`-drive if=pflash,...`).
//!
//! QEMU's `pflash-cfi01` model (as used by `hw/riscv/virt.c`) implements the
//! Intel/Sharp extended command set: commands are single bytes, with no
//! address-based unlock sequence (unlike the AMD/Fujitsu two-cycle unlock
//! sequence used by many other parts). The relevant commands are:
//!
//! - `0x40`/`0x10`: single word program. Followed by a second write of the
//!   data to the target address.
//! - `0x20`: block (sector) erase. The device erases the entire sector
//!   containing the address the command was written to.
//! - `0xFF`: return to normal "read array" mode.
//!
//! After a program or erase operation completes, the device does *not*
//! automatically return to array-read mode: reads keep returning status
//! register bits until an explicit `0xFF` (or equivalent) is written. This
//! driver always issues that reset after each operation.
//!
//! This behavior was verified empirically against QEMU 11.0.1's `riscv virt`
//! `pflash` device (`sector-length = 0x40000`, `width = 4`,
//! `device-width = 2`), since it differs from the AMD-style command set
//! (`0xAA`/`0x55` unlock, `0xA0` program) that QEMU's device tree /
//! `cfi-flash` compatible string might otherwise suggest.

use core::cell::Cell;
use core::ops::{Index, IndexMut};

use kernel::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::interfaces::{Readable, Writeable};

/// Size, in bytes, of a single erase sector on this device.
///
/// This matches the `sector-length` QEMU configures for the `virt` machine's
/// `pflash` devices (queryable at runtime through the CFI query mode, but
/// hardcoded here as it is fixed for this platform).
pub const SECTOR_SIZE: usize = 256 * 1024;
const SECTOR_WORDS: usize = SECTOR_SIZE / 4;

/// Intel/Sharp command set command values understood by QEMU's
/// `pflash-cfi01` model.
mod cmd {
    pub const PROGRAM: u32 = 0x40;
    pub const ERASE: u32 = 0x20;
    pub const READ_ARRAY: u32 = 0xFF;
}

/// MMIO representation of a `pflash` device, as a flat array of 32-bit-wide
/// words. `WORDS` is the total device size in 32-bit words (e.g. `0x0200_0000
/// / 4` for a 32 MiB bank).
#[repr(C)]
pub struct PflashRegisters<const WORDS: usize> {
    data: [ReadWrite<u32>; WORDS],
}

/// A single erase-sector-sized page of flash, as required by
/// [`hil::flash::Flash`].
///
/// An example instantiation looks like:
///
/// ```rust, ignore
/// # use kernel::static_init;
/// # use qemu_virt_chip::pflash::PflashPage;
/// let pagebuffer = unsafe { static_init!(PflashPage, PflashPage::default()) };
/// ```
pub struct PflashPage(pub [u8; SECTOR_SIZE]);

impl Default for PflashPage {
    // A page here is sector-sized (256 KiB) to match this device's erase
    // granularity, unlike most other `Flash` implementations' much smaller
    // pages. This is only ever constructed once into `static` storage via
    // `static_init!`, never actually placed on a live call stack.
    #[allow(clippy::large_stack_arrays, clippy::large_stack_frames)]
    fn default() -> Self {
        Self([0; SECTOR_SIZE])
    }
}

impl PflashPage {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<usize> for PflashPage {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for PflashPage {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for PflashPage {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Tracks the current state and command of the flash, mirroring the
/// convention used by other synchronous-hardware `Flash` implementations
/// (e.g. the nRF52 NVMC driver): the operation itself completes
/// synchronously, and a deferred call is used to issue the client callback
/// from a fresh call stack.
#[derive(Clone, Copy, PartialEq)]
enum FlashState {
    Ready,
    Read,
    Write,
    Erase,
}

pub struct Pflash<'a, const WORDS: usize> {
    registers: StaticRef<PflashRegisters<WORDS>>,
    client: OptionalCell<&'a dyn hil::flash::Client<Pflash<'a, WORDS>>>,
    buffer: TakeCell<'static, PflashPage>,
    state: Cell<FlashState>,
    deferred_call: DeferredCall,
}

impl<const WORDS: usize> Pflash<'_, WORDS> {
    pub fn new(registers: StaticRef<PflashRegisters<WORDS>>) -> Self {
        Self {
            registers,
            client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            state: Cell::new(FlashState::Ready),
            deferred_call: DeferredCall::new(),
        }
    }

    /// Total device size, in 32-bit words.
    fn num_words(&self) -> usize {
        WORDS
    }

    /// Number of erase sectors on this device.
    fn num_sectors(&self) -> usize {
        self.num_words() / SECTOR_WORDS
    }

    /// Issue a command word, then wait for the device to leave its "busy"
    /// (I/O access) mode.
    ///
    /// QEMU's model completes program/erase operations synchronously within
    /// the write that triggers them, so this never actually spins, but
    /// mirrors what a driver for real CFI hardware would need to do (poll
    /// the status register until the "ready" bit is set) and keeps this
    /// implementation robust if that assumption changes.
    fn command(&self, word_index: usize, value: u32) {
        self.registers.data[word_index].set(value);
    }

    /// Return the device to normal "read array" mode. Necessary after every
    /// program or erase operation: QEMU's model does not do this
    /// automatically, and reads would otherwise keep returning status
    /// register contents instead of flash contents.
    fn read_array_mode(&self) {
        self.command(0, cmd::READ_ARRAY);
    }

    fn is_page_blank(&self, page_number: usize) -> bool {
        let start = page_number * SECTOR_WORDS;
        (start..start + SECTOR_WORDS).all(|i| self.registers.data[i].get() == 0xFFFF_FFFF)
    }

    fn erase_sector(&self, page_number: usize) {
        let start = page_number * SECTOR_WORDS;
        self.command(start, cmd::ERASE);
        self.read_array_mode();
    }

    fn program_word(&self, word_index: usize, value: u32) {
        self.command(word_index, cmd::PROGRAM);
        self.command(word_index, value);
        self.read_array_mode();
    }

    fn read_range(
        &self,
        page_number: usize,
        buffer: &'static mut PflashPage,
    ) -> Result<(), (ErrorCode, &'static mut PflashPage)> {
        if page_number >= self.num_sectors() {
            return Err((ErrorCode::INVAL, buffer));
        }

        let start = page_number * SECTOR_WORDS;
        for i in 0..(buffer.len() / 4) {
            let word = self.registers.data[start + i].get();
            let bytes = word.to_le_bytes();
            buffer[i * 4] = bytes[0];
            buffer[i * 4 + 1] = bytes[1];
            buffer[i * 4 + 2] = bytes[2];
            buffer[i * 4 + 3] = bytes[3];
        }

        self.buffer.replace(buffer);
        self.state.set(FlashState::Read);
        self.deferred_call.set();

        Ok(())
    }

    fn write_page_impl(
        &self,
        page_number: usize,
        data: &'static mut PflashPage,
    ) -> Result<(), (ErrorCode, &'static mut PflashPage)> {
        if page_number >= self.num_sectors() {
            return Err((ErrorCode::INVAL, data));
        }

        if !self.is_page_blank(page_number) {
            self.erase_sector(page_number);
        }

        let start = page_number * SECTOR_WORDS;
        for i in 0..(data.len() / 4) {
            let word = u32::from_le_bytes([
                data[i * 4],
                data[i * 4 + 1],
                data[i * 4 + 2],
                data[i * 4 + 3],
            ]);
            self.program_word(start + i, word);
        }

        self.buffer.replace(data);
        self.state.set(FlashState::Write);
        self.deferred_call.set();

        Ok(())
    }

    fn erase_page_impl(&self, page_number: usize) -> Result<(), ErrorCode> {
        if page_number >= self.num_sectors() {
            return Err(ErrorCode::INVAL);
        }

        if !self.is_page_blank(page_number) {
            self.erase_sector(page_number);
        }

        self.state.set(FlashState::Erase);
        self.deferred_call.set();

        Ok(())
    }

    fn handle_interrupt(&self) {
        let state = self.state.get();
        self.state.set(FlashState::Ready);

        match state {
            FlashState::Read => {
                self.client.map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.read_complete(buffer, Ok(()));
                    });
                });
            }
            FlashState::Write => {
                self.client.map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.write_complete(buffer, Ok(()));
                    });
                });
            }
            FlashState::Erase => {
                self.client.map(|client| {
                    client.erase_complete(Ok(()));
                });
            }
            FlashState::Ready => {}
        }
    }
}

impl<'a, const WORDS: usize, C: hil::flash::Client<Pflash<'a, WORDS>>>
    hil::flash::HasClient<'a, C> for Pflash<'a, WORDS>
{
    fn set_client(&'a self, client: &'a C) {
        self.client.set(client);
    }
}

impl<const WORDS: usize> hil::flash::Flash for Pflash<'_, WORDS> {
    type Page = PflashPage;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        self.read_range(page_number, buf)
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        self.write_page_impl(page_number, buf)
    }

    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode> {
        self.erase_page_impl(page_number)
    }
}

impl<const WORDS: usize> DeferredCallClient for Pflash<'_, WORDS> {
    fn handle_deferred_call(&self) {
        self.handle_interrupt();
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
