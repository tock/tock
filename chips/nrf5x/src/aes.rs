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

use core::cell::Cell;
use kernel::hil::symmetric_encryption;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
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

#[derive(Copy, Clone, Debug)]
enum AESMode {
    ECB,
    CTR,
    CBC,
}

pub struct AesECB<'a> {
    registers: StaticRef<AesEcbRegisters>,
    mode: Cell<AESMode>,
    client: OptionalCell<&'a dyn kernel::hil::symmetric_encryption::Client<'a>>,
    /// Input either plaintext or ciphertext to be encrypted or decrypted.
    input: TakeCell<'static, [u8]>,
    output: TakeCell<'static, [u8]>,
    current_idx: Cell<usize>,
    start_idx: Cell<usize>,
    end_idx: Cell<usize>,
}

impl<'a> AesECB<'a> {
    pub fn new() -> AesECB<'a> {
        AesECB {
            registers: AESECB_BASE,
            mode: Cell::new(AESMode::CTR),
            client: OptionalCell::empty(),
            input: TakeCell::empty(),
            output: TakeCell::empty(),
            current_idx: Cell::new(0),
            start_idx: Cell::new(0),
            end_idx: Cell::new(0),
        }
    }

    fn set_dma(&self) {
        unsafe {
            self.registers.ecbdataptr.set(ECB_DATA.as_ptr() as u32);
        }
    }

    /// Verify that the provided start and stop indices work with the given
    /// buffers.
    fn try_set_indices(&self, start_index: usize, stop_index: usize) -> bool {
        stop_index.checked_sub(start_index).map_or(false, |sublen| {
            sublen % symmetric_encryption::AES128_BLOCK_SIZE == 0 && {
                self.input.map_or_else(
                    || {
                        // The destination buffer is also the input
                        if self.output.map_or(false, |dest| stop_index <= dest.len()) {
                            self.current_idx.set(0);
                            self.start_idx.set(start_index);
                            self.end_idx.set(stop_index);
                            true
                        } else {
                            false
                        }
                    },
                    |source| {
                        if sublen == source.len()
                            && self.output.map_or(false, |dest| stop_index <= dest.len())
                        {
                            // We will start writing to the AES from the
                            // beginning of `source`, and end at its end
                            self.current_idx.set(0);

                            // We will start reading from the AES into `dest` at
                            // `start_index`, and continue until `stop_index`
                            self.start_idx.set(start_index);
                            self.end_idx.set(stop_index);
                            true
                        } else {
                            false
                        }
                    },
                )
            }
        })
    }

    // FIXME: should this be performed in constant time i.e. skip the break part
    // and always loop 16 times?
    fn update_ctr(&self) {
        for i in (PLAINTEXT_START..PLAINTEXT_END).rev() {
            unsafe {
                ECB_DATA[i] += 1;
                if ECB_DATA[i] != 0 {
                    break;
                }
            }
        }
    }

    /// Get the relevant positions of our input data whether we are using a
    /// source buffer or overwriting the destination buffer.
    fn get_start_end_take(&self) -> (usize, usize, usize) {
        let current_idx = self.current_idx.get();

        // Location in the appropriate source buffer we are currently working
        // on.
        let start = current_idx + self.input.map_or(self.start_idx.get(), |_| 0);
        // Last index in the appropriate source buffer we need to work on.
        let end = self.end_idx.get() - self.input.map_or(0, |_| self.start_idx.get());

        // Get the number of bytes that were used in the keystream/block.
        let take = match end.checked_sub(start) {
            Some(v) if v > symmetric_encryption::AES128_BLOCK_SIZE => {
                symmetric_encryption::AES128_BLOCK_SIZE
            }
            Some(v) => v,
            None => 0,
        };

        (start, end, take)
    }

    fn copy_plaintext(&self) {
        let (start, _end, take) = self.get_start_end_take();

        // Copy the plaintext either from the source if it exists or from the
        // destination buffer.
        if take > 0 {
            match self.mode.get() {
                AESMode::ECB => {
                    self.input.map_or_else(
                        || {
                            self.output.map(|output| {
                                for i in 0..take {
                                    unsafe {
                                        ECB_DATA[i + PLAINTEXT_START] = output[i + start];
                                    }
                                }
                            });
                        },
                        |input| {
                            for i in 0..take {
                                unsafe {
                                    ECB_DATA[i + PLAINTEXT_START] = input[i + start];
                                }
                            }
                        },
                    );
                }

                AESMode::CBC => {
                    self.input.map_or_else(
                        || {
                            self.output.map(|output| {
                                for i in 0..take {
                                    let ecb_idx = i + PLAINTEXT_START;

                                    unsafe {
                                        ECB_DATA[ecb_idx] = ECB_DATA[ecb_idx] ^ output[i + start];
                                    }
                                }
                            });
                        },
                        |input| {
                            for i in 0..take {
                                let ecb_idx = i + PLAINTEXT_START;
                                unsafe {
                                    ECB_DATA[ecb_idx] = ECB_DATA[ecb_idx] ^ input[i + start];
                                }
                            }
                        },
                    );
                }

                AESMode::CTR => {
                    // no copying plaintext in ctr mode
                }
            }
        }
    }

    fn crypt(&self) {
        match self.mode.get() {
            AESMode::CTR => {}
            AESMode::ECB => {
                // Need to copy the plaintext to the ECB buffer.
                self.copy_plaintext();
            }
            AESMode::CBC => {
                self.copy_plaintext();
            }
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
            let (start, end, take) = self.get_start_end_take();
            let start_idx = self.start_idx.get();
            let current_idx = self.current_idx.get();

            match self.mode.get() {
                AESMode::CTR => {
                    // Fill in the ciphertext in the output buffer.
                    if take > 0 {
                        self.input.map_or_else(
                            || {
                                // No input buffer, so source data comes from
                                // output buffer.
                                self.output.map(|output| {
                                    for i in 0..take {
                                        let in_byte = output[start + i];
                                        let keystream_byte = unsafe { ECB_DATA[i + PLAINTEXT_END] };

                                        output[start + i] = keystream_byte ^ in_byte;
                                    }
                                });
                            },
                            |input| {
                                self.output.map(|output| {
                                    let start_idx = self.start_idx.get();

                                    for i in 0..take {
                                        let in_byte = input[start + i];
                                        let keystream_byte = unsafe { ECB_DATA[i + PLAINTEXT_END] };

                                        output[start_idx + current_idx + i] =
                                            keystream_byte ^ in_byte;
                                    }
                                });
                            },
                        );

                        self.update_ctr();
                    }
                }

                AESMode::ECB => {
                    // Copy ciphertext to output.
                    if take > 0 {
                        self.output.map(|output| {
                            for i in 0..take {
                                // We write to the buffer starting at the
                                // originally provided start index, plus our
                                // offset at current_idx.
                                let dest_idx = start_idx + current_idx + i;
                                unsafe {
                                    output[dest_idx] = ECB_DATA[i + PLAINTEXT_END];
                                }
                            }
                        });
                    }
                }
                AESMode::CBC => {
                    // Copy ciphertext to both output AND the ECB payload to use
                    // on the next iteration.
                    if take > 0 {
                        self.output.map(|output| {
                            for i in 0..take {
                                // We write to the buffer starting at the
                                // originally provided start index, plus our
                                // offset at current_idx.
                                let dest_idx = start_idx + current_idx + i;
                                unsafe {
                                    output[dest_idx] = ECB_DATA[i + PLAINTEXT_END];
                                    ECB_DATA[i + PLAINTEXT_START] = ECB_DATA[i + PLAINTEXT_END];
                                }
                            }
                        });
                    }
                }
            }

            // Advance through the buffer.
            self.current_idx.set(current_idx + take);

            // Check if we are done or if we need to crypt another block.
            if start + take < end {
                // More to do.
                self.crypt();
            } else {
                self.output.take().map(|output| {
                    self.client
                        .map(move |client| client.crypt_done(self.input.take(), output));
                });
            }
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
    fn enable(&self) {
        self.set_dma();
    }

    fn disable(&self) {
        self.registers.task_stopecb.write(Task::ENABLE::CLEAR);
        self.disable_interrupts();
    }

    fn set_client(&'a self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if key.len() != symmetric_encryption::AES128_KEY_SIZE {
            Err(ErrorCode::INVAL)
        } else {
            for (i, c) in key.iter().enumerate() {
                unsafe {
                    ECB_DATA[i] = *c;
                }
            }
            Ok(())
        }
    }

    fn set_iv(&self, iv: &[u8]) -> Result<(), ErrorCode> {
        if iv.len() != symmetric_encryption::AES128_BLOCK_SIZE {
            Err(ErrorCode::INVAL)
        } else {
            for (i, c) in iv.iter().enumerate() {
                unsafe {
                    ECB_DATA[i + PLAINTEXT_START] = *c;
                }
            }
            Ok(())
        }
    }

    // not needed by NRF5x
    fn start_message(&self) {
        ()
    }

    fn crypt(
        &self,
        source: Option<&'static mut [u8]>,
        dest: &'static mut [u8],
        start_index: usize,
        stop_index: usize,
    ) -> Option<(
        Result<(), ErrorCode>,
        Option<&'static mut [u8]>,
        &'static mut [u8],
    )> {
        self.input.put(source);
        self.output.replace(dest);
        if self.try_set_indices(start_index, stop_index) {
            self.crypt();
            None
        } else {
            Some((
                Err(ErrorCode::INVAL),
                self.input.take(),
                self.output.take().unwrap(),
            ))
        }
    }
}

impl kernel::hil::symmetric_encryption::AES128ECB for AesECB<'_> {
    // not needed by NRF5x (the configuration is the same for encryption and decryption)
    fn set_mode_aes128ecb(&self, encrypting: bool) -> Result<(), ErrorCode> {
        if encrypting {
            self.mode.set(AESMode::ECB);
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }
}

impl kernel::hil::symmetric_encryption::AES128Ctr for AesECB<'_> {
    // not needed by NRF5x (the configuration is the same for encryption and decryption)
    fn set_mode_aes128ctr(&self, _encrypting: bool) -> Result<(), ErrorCode> {
        self.mode.set(AESMode::CTR);
        Ok(())
    }
}

impl kernel::hil::symmetric_encryption::AES128CBC for AesECB<'_> {
    fn set_mode_aes128cbc(&self, encrypting: bool) -> Result<(), ErrorCode> {
        if encrypting {
            self.mode.set(AESMode::CBC);
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }
}

//TODO: replace this placeholder with a proper implementation of the AES system
impl<'a> kernel::hil::symmetric_encryption::AES128CCM<'a> for AesECB<'a> {
    /// Set the client instance which will receive `crypt_done()` callbacks
    fn set_client(&'a self, _client: &'a dyn kernel::hil::symmetric_encryption::CCMClient) {}

    /// Set the key to be used for CCM encryption
    fn set_key(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Ok(())
    }

    /// Set the nonce (length NONCE_LENGTH) to be used for CCM encryption
    fn set_nonce(&self, _nonce: &[u8]) -> Result<(), ErrorCode> {
        Ok(())
    }

    /// Try to begin the encryption/decryption process
    fn crypt(
        &self,
        _buf: &'static mut [u8],
        _a_off: usize,
        _m_off: usize,
        _m_len: usize,
        _mic_len: usize,
        _confidential: bool,
        _encrypting: bool,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        Ok(())
    }
}
