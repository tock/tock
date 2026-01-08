// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// 2 options: AESCTR fully in sw or AES CTR that uses
// some block cipher in hw.

use core::cmp::min;
use kernel::utilities::copy_slice::CopyOrErr;
use kernel::utilities::leasable_buffer::SubSliceMut;
// HW backed
use kernel::hil::symmetric_encryption::{self, AES128_BLOCK_SIZE};
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::ErrorCode;

use ctr::cipher::{KeyIvInit, StreamCipher};

type Aes128Ctr128LE = ctr::Ctr128LE<aes::Aes128>;

type AESKey = [u8; 16]; // AES-128 key size
type AESIv = [u8; 16]; // AES-128 IV size

/// Fully software implementation of AES-128 CTR mode using
/// the RustCrypto library. This implementation does not use
/// any hardware acceleration and is suitable for platforms
/// that do not have AES hardware support or where the hardware
/// support is not available.
///
/// This uses the `ctr` and `aes` crates from RustCrypto to expose AES128Ctr support.
struct Aes128CtrSw<'a> {
    client: OptionalCell<&'a dyn symmetric_encryption::Client<'a>>,
    iv: TakeCell<'a, AESIv>,   // Initialization vector
    key: TakeCell<'a, AESKey>, // AES key
}

impl<'a> Aes128CtrSw<'a> {
    pub fn new(iv: &'a mut AESIv, key: &'a mut AESKey) -> Aes128CtrSw<'a> {
        Aes128CtrSw {
            client: OptionalCell::empty(),
            iv: TakeCell::new(iv),
            key: TakeCell::new(key),
        }
    }
}

impl<'a> capsules_core::aes::Aes128Ctr<'a> for Aes128CtrSw<'a> {
    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.client.set(client)
    }

    fn setup_cipher(
        &self,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
        iv: &[u8],
    ) -> Result<(), ErrorCode> {
        // (TODO) IV length check
        let iv: &[u8; symmetric_encryption::AES128_BLOCK_SIZE] =
            iv.try_into().map_err(|_| ErrorCode::INVAL)?;

        self.iv.map(|iv_buf| iv_buf.copy_from_slice(iv));
        self.key.map(|key_buf| key_buf.copy_from_slice(key));
        Ok(())
    }

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
        let mut local_key = [0u8; symmetric_encryption::AES128_KEY_SIZE];
        let mut local_iv = [0u8; symmetric_encryption::AES128_BLOCK_SIZE];

        self.key.map(|key_buf| local_key.copy_from_slice(key_buf));
        self.iv.map(|iv_buf| local_iv.copy_from_slice(iv_buf));

        // Create cipher to be used from key and IV.
        let mut cipher = Aes128Ctr128LE::new(&local_key.into(), &local_iv.into());

        // If the source is Some, copy the keystream to dest.
        // If source is None, encrypt in place the dest buf.
        let source = if let Some(src_buf) = source {
            // `crypt` documentation states that the src and dst
            // buffers must be the same length, so error if they are not.
            if let Err(_) = dest
                .as_mut_slice()
                .copy_from_slice_or_err(src_buf.as_slice())
            {
                return Err((ErrorCode::INVAL, Some(src_buf), dest));
            } else {
                // Since we bind/move src_buf, return src_buf
                // so we can keep using source later.
                Some(src_buf)
            }
        } else {
            None
        };

        if let Err(_) = cipher.try_apply_keystream(dest.as_mut_slice()) {
            // If the keystream could not be applied, return an error.
            return Err((ErrorCode::INVAL, source, dest));
        };

        // Finished crypto operation since the sw crypto
        // func in RustCrypto are blocking and sync.
        self.client.map(|client| {
            client.crypt_done(source, Ok(dest));
        });

        Ok(())
    }
}

/// Hardware-backed AES-128 CTR mode implementation.
/// This implementation uses a generic AES hardware
/// ECB cipher to perform the CTR mode encryption/decryption.
pub struct Aes128CtrEcbBase<'a, ECB: capsules_core::aes::Aes128Ecb<'a>> {
    client: OptionalCell<&'a dyn symmetric_encryption::Client<'a>>,
    counter_block: TakeCell<'static, [u8; AES128_BLOCK_SIZE]>, // Initialization vector
    plaintext: MapCell<SubSliceMut<'static, u8>>,              // Buffer for plaintext
    aes_ecb: &'a ECB,
}

// (todo) is this the buf size we want to use?
impl<'a, ECB> Aes128CtrEcbBase<'a, ECB>
where
    ECB: capsules_core::aes::Aes128Ecb<'a>,
{
    pub fn new(
        aes_ecb: &'a ECB,
        buf: &'static mut [u8; symmetric_encryption::AES128_BLOCK_SIZE],
    ) -> Aes128CtrEcbBase<'a, ECB> {
        Aes128CtrEcbBase {
            client: OptionalCell::empty(),
            counter_block: TakeCell::new(buf),
            plaintext: MapCell::empty(),
            aes_ecb,
        }
    }

    // TODO: Add cargo test for this function.
    /// Helper method to update the counter block, a 16byte block.
    ///
    /// We start from the least significat byte (little endian)
    /// and increment it, propagating any carry on overflow to
    /// the next byte.
    ///
    /// We need each counter block to be unique. We take our 16-byte
    /// counter block and increment it by 1 each time we encrypt a block.
    /// This allows us to encrypt 2^128 counter blocks per IV. It is
    /// good practice to use a unique IV for each message.
    fn update_ctr_le(&self, ctr_block: &mut [u8; symmetric_encryption::AES128_BLOCK_SIZE]) {
        for i in 0..symmetric_encryption::AES128_BLOCK_SIZE {
            let (res, carry) = ctr_block[i].overflowing_add(1);
            ctr_block[i] = res;
            if !carry {
                // If there was no carry, we can stop incrementing.
                break;
            }
        }
    }

    /// Helper method to perform block XOR for CTR.
    ///
    /// XOR plaintext up to but not past the AES128 block size.
    fn perform_xor(
        &self,
        plaintext: &mut [u8],
        ctr_block: &[u8; symmetric_encryption::AES128_BLOCK_SIZE],
    ) {
        // XOR the plaintext with the counter block to produce the ciphertext.
        for i in 0..(min(plaintext.len(), symmetric_encryption::AES128_BLOCK_SIZE)) {
            plaintext[i] ^= ctr_block[i];
        }
    }
}

impl<'a, ECB> capsules_core::aes::Aes128Ctr<'a> for Aes128CtrEcbBase<'a, ECB>
where
    ECB: capsules_core::aes::Aes128Ecb<'a>,
{
    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.client.set(client);
    }

    fn setup_cipher(
        &self,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
        iv: &[u8],
    ) -> Result<(), ErrorCode> {
        // Set the key and IV for AES128CTR operation.
        // The IV can be of variable length, and must be
        // checked by the implementor to ensure it is
        // valid.
        self.aes_ecb.setup_cipher(key)?;

        self.counter_block
            .map(|ctr_block| {
                // Copy the IV into the counter block.
                ctr_block.as_mut_slice().copy_from_slice(iv);
                Ok(())
            })
            .map_or(Err(ErrorCode::NODEVICE), |result| result)
    }

    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            kernel::ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    > {
        // Obtain CTR block and perform ECB.
        match self.counter_block.take() {
            Some(ctr_block) => {
                // We must store the source or destination buffer since
                // we will only perform the CTR XOR after the ECB crypt
                // operation is complete.
                self.plaintext.put(dest);
                self.aes_ecb.crypt(None, SubSliceMut::new(ctr_block))
            }
            None => return Err((ErrorCode::NODEVICE, source, dest)),
        }
    }
}

impl<'a, ECB> symmetric_encryption::Client<'a> for Aes128CtrEcbBase<'a, ECB>
where
    ECB: capsules_core::aes::Aes128Ecb<'a>,
{
    fn crypt_done(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        result: Result<SubSliceMut<'static, u8>, (kernel::ErrorCode, SubSliceMut<'static, u8>)>,
    ) {
        // ECB completed operation on counter block. Update the counter
        // block and replace.
        let _ = match result {
            Ok(mut ctr_block) => {
                let ctr_block_slice: &mut [u8; symmetric_encryption::AES128_BLOCK_SIZE] = ctr_block
                    .as_mut_slice()
                    .try_into()
                    .expect("ctr_block returned here must be 16 bytes");

                let mut completed_ctr_op = false;
                // XOR with the first 16 bytes of the plaintext
                self.plaintext.map(|plaintext| {
                    // Obtain plaintext to XOR with the counter block.

                    self.perform_xor(plaintext.as_mut_slice(), ctr_block_slice);

                    // Shrink buffer to remaining data to be processed.
                    plaintext.slice(symmetric_encryption::AES128_BLOCK_SIZE..);

                    // Determine if there are more blocks to encrypt/decrypt.
                    if plaintext.len() == 0 {
                        completed_ctr_op = true;
                    }
                });

                if completed_ctr_op {
                    // If we are done with processing all blocks, notify the CTR client.
                    self.client.map(|client| {
                        if let Some(dest_buf) = self.plaintext.take() {
                            client.crypt_done(source, Ok(dest_buf));
                        };
                    });
                    Ok(())
                } else {
                    // There are more blocks to process. Increment the counter then
                    // continue.
                    self.update_ctr_le(ctr_block_slice);
                    self.aes_ecb.crypt(None, ctr_block)
                }
            }
            Err((error_code, dest_buf)) => Err((error_code, source, dest_buf)),
        }
        .map_err(|(error_code, src, dest)| {
            // notify client of the error
            self.client.map(|client| {
                client.crypt_done(src, Err((error_code, dest)));
            });
        });
    }
}
