// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! AES128 driver, nRF5X-family
//!
//! Provides a simple driver to encrypt and decrypt messages using aes128-ctr
//! mode on top of aes128-ecb, as well as encrypt with aes128-ecb and
//! aes128-cbc.
//!
//! Roughly, the module uses three buffers with the following content:
//!
//! * Key
//! * Initial counter
//! * Payload, to be encrypted or decrypted
//!
//! ### Key
//! The key is used for getting a key and configure it in the AES chip
//!
//! ### Initial Counter
//! Counter to be used for aes-ctr and it is entered into AES to generate the
//! the keystream. After each encryption the initial counter is incremented
//!
//! ### Payload
//! Data to be encrypted or decrypted it is XOR:ed with the generated keystream
//!
//! ### Things to highlight that can be improved:
//!
//! * ECB_DATA must be a static mut \[u8\] and can't be located in the struct
//! * PAYLOAD size is restricted to 128 bytes
//!
//! Authors
//! --------
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: April 21, 2017

use core::ptr::addr_of;
use core::u8;
use kernel::hil::symmetric_encryption;
use kernel::hil::symmetric_encryption::AES128_BLOCK_SIZE;
use kernel::utilities::cells::MapCell;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

// DMA buffer that the aes chip will mutate during encryption
// Byte 0-15   - Key
// Byte 16-32  - Payload
// Byte 33-47  - Ciphertext
static mut ECB_DATA: [u8; 48] = [0; 48];

#[allow(dead_code)]
const KEY_START: usize = 0;
#[allow(dead_code)]
const KEY_END: usize = 15;
const PLAINTEXT_START: usize = 16;
const PLAINTEXT_END: usize = 32;
#[allow(dead_code)]
const CIPHERTEXT_START: usize = 33;
#[allow(dead_code)]
const CIPHERTEXT_END: usize = 47;

const AESECB_BASE: StaticRef<AesEcbRegisters> =
    unsafe { StaticRef::new(0x4000E000 as *const AesEcbRegisters) };

#[repr(C)]
struct AesEcbRegisters {
    /// Start ECB block encrypt
    /// - Address 0x000 - 0x004
    task_startecb: WriteOnly<u32, Task::Register>,
    /// Abort a possible executing ECB operation
    /// - Address: 0x004 - 0x008
    task_stopecb: WriteOnly<u32, Task::Register>,
    /// Reserved
    _reserved1: [u32; 62],
    /// ECB block encrypt complete
    /// - Address: 0x100 - 0x104
    event_endecb: ReadWrite<u32, Event::Register>,
    /// ECB block encrypt aborted because of a STOPECB task or due to an error
    /// - Address: 0x104 - 0x108
    event_errorecb: ReadWrite<u32, Event::Register>,
    /// Reserved
    _reserved2: [u32; 127],
    /// Enable interrupt
    /// - Address: 0x304 - 0x308
    intenset: ReadWrite<u32, Intenset::Register>,
    /// Disable interrupt
    /// - Address: 0x308 - 0x30c
    intenclr: ReadWrite<u32, Intenclr::Register>,
    /// Reserved
    _reserved3: [u32; 126],
    /// ECB block encrypt memory pointers
    /// - Address: 0x504 - 0x508
    ecbdataptr: ReadWrite<u32, EcbDataPointer::Register>,
}

register_bitfields! [u32,
    /// Start task
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Read event
    Event [
        READY OFFSET(0) NUMBITS(1)
    ],

    /// Enabled interrupt
    Intenset [
        ENDECB OFFSET(0) NUMBITS(1),
        ERRORECB OFFSET(1) NUMBITS(1)
    ],

    /// Disable interrupt
    Intenclr [
        ENDECB OFFSET(0) NUMBITS(1),
        ERRORECB OFFSET(1) NUMBITS(1)
    ],

    /// ECB block encrypt memory pointers
    EcbDataPointer [
        POINTER OFFSET(0) NUMBITS(32)
    ]
];

pub struct AesECB<'a> {
    registers: StaticRef<AesEcbRegisters>,
    client: OptionalCell<&'a dyn kernel::hil::symmetric_encryption::Client<'a>>,
    /// Input either plaintext or ciphertext to be encrypted or decrypted.
    source_buf: MapCell<SubSliceMut<'static, u8>>,
    dest_buf: MapCell<SubSliceMut<'static, u8>>,
}

impl<'a> AesECB<'a> {
    pub const fn new() -> AesECB<'a> {
        AesECB {
            registers: AESECB_BASE,
            client: OptionalCell::empty(),
            source_buf: MapCell::empty(),
            dest_buf: MapCell::empty(),
        }
    }

    fn set_dma(&self) {
        self.registers.ecbdataptr.set(addr_of!(ECB_DATA) as u32);
    }

    // Begin crypt operation on a buffer.
    fn start_crypt(&self, plaintext_block: &[u8; symmetric_encryption::AES128_BLOCK_SIZE]) {
        // Copy the plaintext to the ECB buffer.
        // This is unsound
        unsafe {
            ECB_DATA[PLAINTEXT_START..PLAINTEXT_END].copy_from_slice(plaintext_block);
        }

        self.registers.event_endecb.write(Event::READY::CLEAR);
        self.registers.task_startecb.set(1);

        self.enable_interrupts();
    }

    /// AesEcb Interrupt handler
    pub fn handle_interrupt(&self) {
        // disable interrupts
        self.disable_interrupts();

        if self.registers.event_endecb.get() == 1 {
            // Copy from ECB buffer to output buffer.
            self.dest_buf.map(|dest| {
                // unsound
                unsafe {
                    // Copy ciphertext to output.
                    dest.as_mut_slice()[..symmetric_encryption::AES128_BLOCK_SIZE]
                        .copy_from_slice(&ECB_DATA[CIPHERTEXT_START..CIPHERTEXT_END]);
                }

                // Slice the output buffer to the next block.
                dest.slice(symmetric_encryption::AES128_BLOCK_SIZE..);

                // If there is a source buffer, slice to advance to next block.
                self.source_buf
                    .map(|source| {
                        // Slice the source buffer to the next block.
                        source.slice(symmetric_encryption::AES128_BLOCK_SIZE..);

                        // Check if we are done or if we need to crypt another block.
                        if source.len() != 0 {
                            let plaintext_block = source.as_slice()[0..AES128_BLOCK_SIZE]
                                .try_into()
                                .expect("...");

                            self.start_crypt(plaintext_block);
                        }
                    })
                    .or_else(|| {
                        self.dest_buf.map(|dest| {
                            if dest.len() != 0 {
                                let plaintext_block = dest.as_slice()[0..AES128_BLOCK_SIZE]
                                    .try_into()
                                    .expect("...");

                                self.start_crypt(plaintext_block);
                            }
                        })
                    });
            });
        }
    }

    fn enable_interrupts(&self) {
        self.registers
            .intenset
            .write(Intenset::ENDECB::SET + Intenset::ERRORECB::SET);
    }

    fn disable_interrupts(&self) {
        self.registers
            .intenclr
            .write(Intenclr::ENDECB::SET + Intenclr::ERRORECB::SET);
    }
}

impl<'a> kernel::hil::symmetric_encryption::AES128<'a> for AesECB<'a> {
    type T = ();

    fn enable(&self) {}

    fn disable(&self) {
        self.registers.task_stopecb.write(Task::ENABLE::CLEAR);
        self.disable_interrupts();
    }

    fn configure(&self, _typer: Self::T) {
        // nothing todo, hw only supports ecb
    }

    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.client.set(client);
    }

    fn set_key(&self, key: &[u8; symmetric_encryption::AES128_KEY_SIZE]) {
        // (todo) we should update this driver to use takecells. Copying
        // over prior impl for now.

        // Copy the key to the ECB buffer.
        unsafe {
            ECB_DATA[KEY_START..KEY_END].copy_from_slice(key);
        }

        self.set_dma();
    }

    fn set_iv(&self, _iv: &[u8]) -> Result<(), ErrorCode> {
        // ECB only hardware and does not support / use IV.
        Ok(())
    }

    // not needed by NRF5x
    fn start_message(&self) {}

    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        mut dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    > {
        // Check the following:
        //   - source / dest active len equal
        //   - length is multiple of AESBLOCKSIZE
        if let Some(source_subslice) = &source {
            if source_subslice.len() != dest.len() {
                return Err((ErrorCode::INVAL, source, dest));
            }
        }

        if dest.len() % symmetric_encryption::AES128_BLOCK_SIZE == 0 {
            return Err((ErrorCode::INVAL, source, dest));
        }

        // Get the buffer that is to be our plaintext block.
        if let Some(mut src_buf) = source {
            // Get 16 byte plaintext slice from the source buffer.
            if let Ok(plaintext_block) =
                src_buf.as_slice()[0..symmetric_encryption::AES128_BLOCK_SIZE].try_into()
            {
                self.start_crypt(plaintext_block);
                Ok(())
            } else {
                Err((ErrorCode::INVAL, Some(src_buf), dest))
            }
        } else {
            // No src buffer so use the dest buffer.
            if let Ok(plaintext_block) =
                dest.as_slice()[0..symmetric_encryption::AES128_BLOCK_SIZE].try_into()
            {
                self.start_crypt(plaintext_block);
                Ok(())
            } else {
                Err((ErrorCode::INVAL, None, dest))
            }
        }
    }
}

impl<'a> kernel::hil::symmetric_encryption::AES128ECB<'a> for AesECB<'a> {
    fn set_mode_aes128ecb(&self) -> Result<(), ErrorCode> {
        // nop
        Ok(())
    }
}
