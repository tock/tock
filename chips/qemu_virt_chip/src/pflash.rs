// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Driver for QEMU's emulated parallel NOR (CFI) flash, as exposed on the
//! `pflash` devices.
//!
//! QEMU's `pflash-cfi01` model (as used by qemu-rv32-virt) implements the
//! Intel/Sharp extended command set: commands are single bytes, with no
//! address-based unlock sequence.

use core::cell::Cell;
use kernel::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::interfaces::{Readable, Writeable};

/// Intel/Sharp command set command values understood by QEMU's `pflash-cfi01`
/// model.
mod cmd {
    pub const PROGRAM: u32 = 0x40;
    pub const ERASE: u32 = 0x20;
    pub const READ_ARRAY: u32 = 0xFF;
}

/// Tracks the current state and command of the flash.
#[derive(Clone, Copy, PartialEq)]
enum FlashState {
    Ready,
    Read,
    Write,
    Erase,
}

/// Driver for a QEMU `pflash` device.
///
/// - `WORDS`: total device size, in 32-bit words.
/// - `PAGE_WORDS`: erase-sector size, in 32-bit words. Must evenly divide
///   `WORDS`.
/// - `P`: the [`hil::flash::Flash::Page`] type used for this device, which must
///   hold exactly `PAGE_WORDS * 4` bytes.
pub struct Pflash<'a, const WORDS: usize, const PAGE_WORDS: usize, P: 'static + Default> {
    registers: StaticRef<[ReadWrite<u32>; WORDS]>,
    client: OptionalCell<&'a dyn hil::flash::Client<Pflash<'a, WORDS, PAGE_WORDS, P>>>,
    buffer: TakeCell<'static, P>,
    state: Cell<FlashState>,
    deferred_call: DeferredCall,
}

impl<const WORDS: usize, const PAGE_WORDS: usize, P: 'static + Default + AsMut<[u8]>>
    Pflash<'_, WORDS, PAGE_WORDS, P>
{
    pub fn new(registers: StaticRef<[ReadWrite<u32>; WORDS]>) -> Self {
        Self {
            registers,
            client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            state: Cell::new(FlashState::Ready),
            deferred_call: DeferredCall::new(),
        }
    }

    /// Number of erase sectors on this device.
    fn num_sectors(&self) -> usize {
        WORDS / PAGE_WORDS
    }

    fn command(&self, word_index: usize, value: u32) {
        self.registers[word_index].set(value);
    }

    /// Return the device to normal "read array" mode.
    ///
    /// Necessary after every program or erase operation: QEMU's model does not
    /// do this automatically, and reads would otherwise keep returning status
    /// register contents instead of flash contents.
    fn read_array_mode(&self) {
        self.command(0, cmd::READ_ARRAY);
    }

    fn is_page_blank(&self, page_number: usize) -> bool {
        let start = page_number * PAGE_WORDS;
        (start..start + PAGE_WORDS).all(|i| self.registers[i].get() == 0xFFFF_FFFF)
    }

    fn erase_sector(&self, page_number: usize) {
        let start = page_number * PAGE_WORDS;
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
        buffer: &'static mut P,
    ) -> Result<(), (ErrorCode, &'static mut P)> {
        if page_number >= self.num_sectors() {
            return Err((ErrorCode::INVAL, buffer));
        }

        let start = page_number * PAGE_WORDS;
        for (i, word) in buffer
            .as_mut()
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .enumerate()
        {
            *word = self.registers[start + i].get().to_le_bytes();
        }

        self.buffer.replace(buffer);
        self.state.set(FlashState::Read);
        self.deferred_call.set();

        Ok(())
    }

    fn write_page_impl(
        &self,
        page_number: usize,
        data: &'static mut P,
    ) -> Result<(), (ErrorCode, &'static mut P)> {
        if page_number >= self.num_sectors() {
            return Err((ErrorCode::INVAL, data));
        }

        let start = page_number * PAGE_WORDS;

        // First, check if anything we want to write will change non-0xFFFFFFFF
        // words already in flash. If so, we need to do a erase first (like
        // normal flash). However, if we are only writing data that is already
        // erased, don't first do an erase. Writing data on the QEMU pflash is
        // VERY slow. So avoiding extra writes is very worthwhile.
        let mut do_erase = false;
        for (i, word) in data.as_mut().as_chunks::<4>().0.iter().enumerate() {
            let value = u32::from_le_bytes(*word);
            let existing_value = self.registers[start + i].get();

            if existing_value != 0xFFFFFFFF && value != existing_value {
                do_erase = true;
                break;
            }
        }

        if do_erase {
            self.erase_sector(page_number);
        }

        for (i, word) in data.as_mut().as_chunks::<4>().0.iter().enumerate() {
            let value = u32::from_le_bytes(*word);
            let existing_value = self.registers[start + i].get();

            // Skip writing the default value. Also, skip writing values that
            // are already in flash. It is incredibly slow to write every word
            // in the sector.
            if value != 0xFFFFFFFF && value != existing_value {
                self.program_word(start + i, value);
            }
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

impl<
    'a,
    const WORDS: usize,
    const PAGE_WORDS: usize,
    P: 'static + Default + AsMut<[u8]>,
    C: hil::flash::Client<Pflash<'a, WORDS, PAGE_WORDS, P>>,
> hil::flash::HasClient<'a, C> for Pflash<'a, WORDS, PAGE_WORDS, P>
{
    fn set_client(&'a self, client: &'a C) {
        self.client.set(client);
    }
}

impl<const WORDS: usize, const PAGE_WORDS: usize, P: 'static + Default + AsMut<[u8]>>
    hil::flash::Flash for Pflash<'_, WORDS, PAGE_WORDS, P>
{
    type Page = P;

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

impl<const WORDS: usize, const PAGE_WORDS: usize, P: 'static + Default + AsMut<[u8]>>
    DeferredCallClient for Pflash<'_, WORDS, PAGE_WORDS, P>
{
    fn handle_deferred_call(&self) {
        self.handle_interrupt();
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
