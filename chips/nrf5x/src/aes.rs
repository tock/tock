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
//! Authors
//! --------
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: April 21, 2017

use core::cell::Cell;
use kernel::ErrorCode;
use kernel::hil::symmetric_encryption;
use kernel::hil::symmetric_encryption::AES128;
use kernel::utilities::cells::MapCell;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
pub use nrf5x_unsafe::aes::AesEcbRegisters;
pub use nrf5x_unsafe::aes::AesEcbRegistersManager;

const KEY_START: usize = 0;
const PLAINTEXT_START: usize = 16;
const PLAINTEXT_END: usize = 32;

#[derive(Copy, Clone, Debug)]
enum AESMode {
    ECB,
    CTR,
    CBC,
}

pub struct AesECB<'a> {
    registers: AesEcbRegistersManager,
    mode: Cell<AESMode>,
    /// DMA buffer for the ECB engine. Needed because we need to set the key and
    /// payload in specific bytes, and then read the ciphertext from the same
    /// buffer.
    ///
    /// - Byte 0-15   - Key
    /// - Byte 16-32  - Payload
    /// - Byte 33-47  - Ciphertext
    ecb_data: MapCell<&'static mut [u8]>,
    /// Input plaintext or ciphertext. SubSliceMut window advances one block at
    /// a time as encryption proceeds. Empty when using in-place (dest-only) mode.
    input: MapCell<SubSliceMut<'static, u8>>,
    /// Output buffer, pre-sliced to the requested start..stop range. Window
    /// advances one block at a time as encryption proceeds.
    output: MapCell<SubSliceMut<'static, u8>>,
    client: OptionalCell<&'a dyn kernel::hil::symmetric_encryption::Client<'a>>,
}

impl AesECB<'_> {
    pub fn new(registers: AesEcbRegistersManager, ecb_data: &'static mut [u8; 48]) -> Self {
        Self {
            registers,
            mode: Cell::new(AESMode::CTR),
            ecb_data: MapCell::new(ecb_data),
            input: MapCell::empty(),
            output: MapCell::empty(),
            client: OptionalCell::empty(),
        }
    }

    /// Returns the number of bytes remaining in the current operation.
    fn remaining_len(&self) -> usize {
        self.input
            .map_or(self.output.map_or(0, |o| o.len()), |i| i.len())
    }

    fn copy_plaintext(&self) {
        fn copy_to_ecb(
            ecb: &MapCell<&'static mut [u8]>,
            buf: &MapCell<SubSliceMut<'static, u8>>,
            len: usize,
        ) {
            ecb.map(|ecb| {
                buf.map(|buf| {
                    ecb[PLAINTEXT_START..PLAINTEXT_START + len].copy_from_slice(&buf[0..len]);
                });
            });
        }

        fn xor_to_ecb(
            ecb: &MapCell<&'static mut [u8]>,
            buf: &MapCell<SubSliceMut<'static, u8>>,
            len: usize,
        ) {
            ecb.map(|ecb| {
                buf.map(|buf| {
                    for i in 0..len {
                        ecb[PLAINTEXT_START + i] ^= buf[i];
                    }
                });
            });
        }

        let take = core::cmp::min(symmetric_encryption::AES_BLOCK_SIZE, self.remaining_len());

        match self.mode.get() {
            AESMode::ECB => {
                // Copy the current plaintext block into the ECB data buffer.
                if self.input.is_some() {
                    copy_to_ecb(&self.ecb_data, &self.input, take);
                } else {
                    copy_to_ecb(&self.ecb_data, &self.output, take);
                }
            }

            AESMode::CBC => {
                // XOR the existing ECB plaintext slot (IV or previous ciphertext)
                // with the current plaintext block.
                if self.input.is_some() {
                    xor_to_ecb(&self.ecb_data, &self.input, take);
                } else {
                    xor_to_ecb(&self.ecb_data, &self.output, take);
                }
            }

            AESMode::CTR => {
                // The counter is already in the ECB plaintext slot; no copy needed.
            }
        }
    }

    /// Start a single ECB encryption block via DMA, preparing the ECB data
    /// buffer with the current plaintext block if needed.
    fn do_crypt(&self) {
        self.copy_plaintext();

        if let Some(ecb_data) = self.ecb_data.take() {
            let _ = self.registers.start_ecb_dma(ecb_data);
        }

        self.enable_interrupts();
    }

    // FIXME: should this be performed in constant time i.e. skip the break part
    // and always loop 16 times?
    fn update_ctr(&self) {
        self.ecb_data.map(|buf| {
            for i in (PLAINTEXT_START..PLAINTEXT_END).rev() {
                buf[i] = buf[i].wrapping_add(1);
                if buf[i] != 0 {
                    break;
                }
            }
        });
    }

    /// AesEcb Interrupt handler
    pub fn handle_interrupt(&self) {
        self.disable_interrupts();

        if self
            .registers
            .registers
            .event_endecb
            .is_set(nrf5x_unsafe::aes::Event::READY)
        {
            // Recover the ECB data buffer from the DMA manager.
            if let Some(ecb_data) = self.registers.finish_ecb_dma() {
                self.ecb_data.replace(ecb_data);
            }

            let take = core::cmp::min(symmetric_encryption::AES_BLOCK_SIZE, self.remaining_len());

            if take > 0 {
                match self.mode.get() {
                    AESMode::ECB => {
                        // Copy ciphertext from the ECB output slot to the output buffer.
                        self.ecb_data.map(|ecb| {
                            self.output.map(|output| {
                                output[0..take]
                                    .copy_from_slice(&ecb[PLAINTEXT_END..PLAINTEXT_END + take]);
                            });
                        });
                    }

                    AESMode::CBC => {
                        // Copy ciphertext to output and save it in the ECB plaintext
                        // slot for CBC chaining on the next block.
                        self.ecb_data.map(|ecb| {
                            self.output.map(|output| {
                                for i in 0..take {
                                    let byte = ecb[PLAINTEXT_END + i];
                                    output[i] = byte;
                                    ecb[PLAINTEXT_START + i] = byte;
                                }
                            });
                        });
                    }

                    AESMode::CTR => {
                        // XOR keystream output with plaintext to produce ciphertext.
                        if self.input.is_some() {
                            self.ecb_data.map(|ecb| {
                                self.input.map(|input| {
                                    self.output.map(|output| {
                                        for i in 0..take {
                                            output[i] = input[i] ^ ecb[PLAINTEXT_END + i];
                                        }
                                    });
                                });
                            });
                        } else {
                            self.ecb_data.map(|ecb| {
                                self.output.map(|output| {
                                    for i in 0..take {
                                        output[i] ^= ecb[PLAINTEXT_END + i];
                                    }
                                });
                            });
                        }
                        self.update_ctr();
                    }
                }

                // Advance both SubSliceMut windows past the block just processed.
                self.output.map(|buf| {
                    buf.slice(take..);
                });
                self.input.map(|buf| {
                    buf.slice(take..);
                });
            }

            if self.remaining_len() > 0 {
                self.do_crypt();
            } else {
                // Recover the original buffers (SubSliceMut::take returns the
                // full underlying &'static mut [u8] regardless of the current
                // window position) and notify the client.
                let input_buf = self.input.take().map(|s| s.take());
                if let Some(output_sub) = self.output.take() {
                    let output = output_sub.take();
                    self.client.map(move |client| {
                        client.crypt_done(input_buf, output);
                    });
                }
            }
        }
    }

    fn enable_interrupts(&self) {
        self.registers.registers.intenset.write(
            nrf5x_unsafe::aes::Intenset::ENDECB::SET + nrf5x_unsafe::aes::Intenset::ERRORECB::SET,
        );
    }

    fn disable_interrupts(&self) {
        self.registers.registers.intenclr.write(
            nrf5x_unsafe::aes::Intenclr::ENDECB::SET + nrf5x_unsafe::aes::Intenclr::ERRORECB::SET,
        );
    }
}

impl<'a> kernel::hil::symmetric_encryption::AES<'a, AES128> for AesECB<'a> {
    fn enable(&self) {}

    fn disable(&self) {
        self.registers.finish_ecb_dma();
        self.disable_interrupts();
    }

    fn set_client(&'a self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if key.len() != symmetric_encryption::AES128_KEY_SIZE {
            Err(ErrorCode::INVAL)
        } else {
            self.ecb_data.map(|buf| {
                buf[KEY_START..KEY_START + symmetric_encryption::AES128_KEY_SIZE]
                    .copy_from_slice(key);
            });
            Ok(())
        }
    }

    fn set_iv(&self, iv: &[u8]) -> Result<(), ErrorCode> {
        if iv.len() != symmetric_encryption::AES_BLOCK_SIZE {
            Err(ErrorCode::INVAL)
        } else {
            self.ecb_data.map(|buf| {
                buf[PLAINTEXT_START..PLAINTEXT_END].copy_from_slice(iv);
            });
            Ok(())
        }
    }

    // not needed by NRF5x
    fn start_message(&self) {}

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
        // Validate indices and buffer sizes before consuming any buffers.
        let len = match stop_index.checked_sub(start_index) {
            Some(l) if l.is_multiple_of(symmetric_encryption::AES_BLOCK_SIZE) => l,
            _ => return Some((Err(ErrorCode::INVAL), source, dest)),
        };

        if stop_index > dest.len() {
            return Some((Err(ErrorCode::INVAL), source, dest));
        }

        if let Some(ref src) = source {
            if src.len() != len {
                return Some((Err(ErrorCode::INVAL), source, dest));
            }
        }

        if let Some(src) = source {
            self.input.replace(SubSliceMut::new(src));
        }

        let mut output_slice = SubSliceMut::new(dest);
        output_slice.slice(start_index..stop_index);
        self.output.replace(output_slice);

        self.do_crypt();
        None
    }
}

impl kernel::hil::symmetric_encryption::AESECB for AesECB<'_> {
    // not needed by NRF5x (the configuration is the same for encryption and decryption)
    fn set_mode_aesecb(&self, encrypting: bool) -> Result<(), ErrorCode> {
        if encrypting {
            self.mode.set(AESMode::ECB);
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }
}

impl kernel::hil::symmetric_encryption::AESCtr for AesECB<'_> {
    // not needed by NRF5x (the configuration is the same for encryption and decryption)
    fn set_mode_aesctr(&self, _encrypting: bool) -> Result<(), ErrorCode> {
        self.mode.set(AESMode::CTR);
        Ok(())
    }
}

impl kernel::hil::symmetric_encryption::AESCBC for AesECB<'_> {
    fn set_mode_aescbc(&self, encrypting: bool) -> Result<(), ErrorCode> {
        if encrypting {
            self.mode.set(AESMode::CBC);
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }
}

//TODO: replace this placeholder with a proper implementation of the AES system
impl<'a> kernel::hil::symmetric_encryption::AESCCM<'a, AES128> for AesECB<'a> {
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
