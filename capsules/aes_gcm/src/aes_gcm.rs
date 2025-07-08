// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Western Digital 2023.

//! Implements an AES-GCM implementation using the underlying
//! AES-CTR implementation.
//!
//! This capsule requires an AES-CTR implementation to support
//! AES-GCM. The implementation relies on AES-CTR, AES-CBC, AES-ECB and
//! AES-CCM to ensure that when this capsule is used it exposes
//! all of supported AES operations in a single API.

use core::cell::Cell;
use ghash::universal_hash::NewUniversalHash;
use ghash::universal_hash::UniversalHash;
use ghash::GHash;
use ghash::Key;
use kernel::hil::symmetric_encryption;
use kernel::hil::symmetric_encryption::{
    AES128Ctr, AES128, AES128CBC, AES128CCM, AES128ECB, AES128_BLOCK_SIZE, AES128_KEY_SIZE,
};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum GCMState {
    Idle,
    GenerateHashKey,
    CtrEncrypt,
}

pub struct Aes128Gcm<'a, A: AES128<'a> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'a>> {
    aes: &'a A,

    mac: OptionalCell<GHash>,

    crypt_buf: TakeCell<'static, [u8]>,

    client: OptionalCell<&'a dyn symmetric_encryption::Client<'a>>,
    ccm_client: OptionalCell<&'a dyn symmetric_encryption::CCMClient>,
    gcm_client: OptionalCell<&'a dyn symmetric_encryption::GCMClient>,

    state: Cell<GCMState>,
    encrypting: Cell<bool>,

    buf: TakeCell<'static, [u8]>,

    pos: Cell<(usize, usize, usize)>,
    key: Cell<[u8; AES128_KEY_SIZE]>,
    iv: Cell<[u8; AES128_KEY_SIZE]>,
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'a>> Aes128Gcm<'a, A> {
    pub fn new(aes: &'a A, crypt_buf: &'static mut [u8]) -> Aes128Gcm<'a, A> {
        Aes128Gcm {
            aes,

            mac: OptionalCell::empty(),

            crypt_buf: TakeCell::new(crypt_buf),

            client: OptionalCell::empty(),
            ccm_client: OptionalCell::empty(),
            gcm_client: OptionalCell::empty(),

            state: Cell::new(GCMState::Idle),
            encrypting: Cell::new(false),

            buf: TakeCell::empty(),
            pos: Cell::new((0, 0, 0)),
            key: Cell::new(Default::default()),
            iv: Cell::new(Default::default()),
        }
    }

    fn start_ctr_encrypt(&self) -> Result<(), ErrorCode> {
        self.aes.set_mode_aes128ctr(self.encrypting.get())?;

        let res = AES128::set_key(self.aes, &self.key.get());
        if res != Ok(()) {
            return res;
        }

        self.aes.set_iv(&self.iv.get()).unwrap();

        self.aes.start_message();
        let crypt_buf = self.crypt_buf.take().unwrap();
        let (_aad_offset, message_offset, message_len) = self.pos.get();

        match AES128::crypt(
            self.aes,
            None,
            crypt_buf,
            message_offset,
            message_offset + message_len + AES128_BLOCK_SIZE,
        ) {
            None => {
                self.state.set(GCMState::CtrEncrypt);
                Ok(())
            }
            Some((res, _, crypt_buf)) => {
                self.crypt_buf.replace(crypt_buf);
                res
            }
        }
    }

    fn crypt_r(
        &self,
        buf: &'static mut [u8],
        aad_offset: usize,
        message_offset: usize,
        message_len: usize,
        encrypting: bool,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.state.get() != GCMState::Idle {
            return Err((ErrorCode::BUSY, buf));
        }

        self.encrypting.set(encrypting);

        self.aes.set_mode_aes128ctr(self.encrypting.get()).unwrap();
        AES128::set_key(self.aes, &self.key.get()).unwrap();
        self.aes.set_iv(&[0; AES128_BLOCK_SIZE]).unwrap();

        self.aes.start_message();
        let crypt_buf = self.crypt_buf.take().unwrap();

        for i in 0..AES128_BLOCK_SIZE {
            crypt_buf[i] = 0;
        }

        match AES128::crypt(self.aes, None, crypt_buf, 0, AES128_BLOCK_SIZE) {
            None => {
                self.state.set(GCMState::GenerateHashKey);
            }
            Some((_res, _, crypt_buf)) => {
                self.crypt_buf.replace(crypt_buf);
            }
        }

        self.buf.replace(buf);
        self.pos.set((aad_offset, message_offset, message_len));
        Ok(())
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'a>>
    symmetric_encryption::CCMClient for Aes128Gcm<'a, A>
{
    fn crypt_done(&self, buf: &'static mut [u8], res: Result<(), ErrorCode>, tag_is_valid: bool) {
        self.ccm_client.map(move |client| {
            client.crypt_done(buf, res, tag_is_valid);
        });
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'a>>
    symmetric_encryption::AES128GCM<'a> for Aes128Gcm<'a, A>
{
    fn set_client(&self, client: &'a dyn symmetric_encryption::GCMClient) {
        self.gcm_client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if key.len() < AES128_KEY_SIZE {
            Err(ErrorCode::INVAL)
        } else {
            let mut new_key = [0u8; AES128_KEY_SIZE];
            new_key.copy_from_slice(key);
            self.key.set(new_key);
            Ok(())
        }
    }

    fn set_iv(&self, nonce: &[u8]) -> Result<(), ErrorCode> {
        let mut new_nonce = [0u8; AES128_KEY_SIZE];
        let len = nonce.len().min(12);

        new_nonce[0..len].copy_from_slice(&nonce[0..len]);
        new_nonce[12..16].copy_from_slice(&[0, 0, 0, 1]);

        self.iv.set(new_nonce);
        Ok(())
    }

    fn crypt(
        &self,
        buf: &'static mut [u8],
        aad_offset: usize,
        message_offset: usize,
        message_len: usize,
        encrypting: bool,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.state.get() != GCMState::Idle {
            return Err((ErrorCode::BUSY, buf));
        }

        let _ = self
            .crypt_r(buf, aad_offset, message_offset, message_len, encrypting)
            .map_err(|(ecode, _)| {
                self.buf.take().map(|buf| {
                    self.gcm_client.map(move |client| {
                        client.crypt_done(buf, Err(ecode), false);
                    });
                });
            });

        Ok(())
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'a>>
    symmetric_encryption::AES128<'a> for Aes128Gcm<'a, A>
{
    fn enable(&self) {
        self.aes.enable();
    }

    fn disable(&self) {
        self.aes.disable();
    }

    fn set_client(&'a self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode> {
        AES128::set_key(self.aes, key)
    }

    fn set_iv(&self, iv: &[u8]) -> Result<(), ErrorCode> {
        self.aes.set_iv(iv)
    }

    fn start_message(&self) {
        self.aes.start_message()
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
        AES128::crypt(self.aes, source, dest, start_index, stop_index)
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'a> + AES128CCM<'a>>
    symmetric_encryption::AES128CCM<'a> for Aes128Gcm<'a, A>
{
    fn set_client(&'a self, client: &'a dyn symmetric_encryption::CCMClient) {
        self.ccm_client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode> {
        AES128CCM::set_key(self.aes, key)
    }

    fn set_nonce(&self, nonce: &[u8]) -> Result<(), ErrorCode> {
        self.aes.set_nonce(nonce)
    }

    fn crypt(
        &self,
        buf: &'static mut [u8],
        a_off: usize,
        m_off: usize,
        m_len: usize,
        mic_len: usize,
        confidential: bool,
        encrypting: bool,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        AES128CCM::crypt(
            self.aes,
            buf,
            a_off,
            m_off,
            m_len,
            mic_len,
            confidential,
            encrypting,
        )
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'a>> AES128Ctr
    for Aes128Gcm<'a, A>
{
    fn set_mode_aes128ctr(&self, encrypting: bool) -> Result<(), ErrorCode> {
        self.aes.set_mode_aes128ctr(encrypting)
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'a>> AES128ECB
    for Aes128Gcm<'a, A>
{
    fn set_mode_aes128ecb(&self, encrypting: bool) -> Result<(), ErrorCode> {
        self.aes.set_mode_aes128ecb(encrypting)
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'a>> AES128CBC
    for Aes128Gcm<'a, A>
{
    fn set_mode_aes128cbc(&self, encrypting: bool) -> Result<(), ErrorCode> {
        self.aes.set_mode_aes128cbc(encrypting)
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'a>>
    symmetric_encryption::Client<'a> for Aes128Gcm<'a, A>
{
    fn crypt_done(&self, _: Option<&'static mut [u8]>, crypt_buf: &'static mut [u8]) {
        match self.state.get() {
            GCMState::Idle => unreachable!(),
            GCMState::GenerateHashKey => {
                let (aad_offset, message_offset, message_len) = self.pos.get();

                let mut mac = GHash::new(Key::from_slice(&crypt_buf[0..AES128_BLOCK_SIZE]));
                let buf = self.buf.take().unwrap();

                if self.encrypting.get() {
                    mac.update_padded(&buf[aad_offset..message_offset]);

                    crypt_buf[AES128_BLOCK_SIZE..(AES128_BLOCK_SIZE + message_len)]
                        .copy_from_slice(&buf[message_offset..(message_offset + message_len)]);
                    for i in 0..AES128_BLOCK_SIZE {
                        crypt_buf[i] = 0;
                    }

                    self.mac.replace(mac);
                } else {
                    let copy_offset = (message_offset / AES128_BLOCK_SIZE) * AES128_BLOCK_SIZE;
                    mac.update_padded(&buf[aad_offset..message_offset]);
                    mac.update_padded(&buf[message_offset..(message_offset + message_len)]);

                    let associated_data_bits = ((message_offset - aad_offset) as u64) * 8;
                    let buffer_bits = (message_len as u64) * 8;

                    let mut block = ghash::Block::default();
                    block[..8].copy_from_slice(&associated_data_bits.to_be_bytes());
                    block[8..].copy_from_slice(&buffer_bits.to_be_bytes());
                    mac.update(&block);

                    let mut tag = mac.finalize().into_bytes();

                    for i in 0..AES128_BLOCK_SIZE {
                        tag[i] ^= crypt_buf[copy_offset + i];
                    }

                    buf[0..AES128_BLOCK_SIZE].copy_from_slice(&tag);
                }
                self.crypt_buf.replace(crypt_buf);
                self.buf.replace(buf);

                self.start_ctr_encrypt().unwrap();
            }
            GCMState::CtrEncrypt => {
                let buf = self.buf.take().unwrap();
                let (aad_offset, message_offset, message_len) = self.pos.get();
                let tag_offset = (message_offset / AES128_BLOCK_SIZE) * AES128_BLOCK_SIZE;
                let copy_offset = (message_offset / AES128_BLOCK_SIZE).max(1) * AES128_BLOCK_SIZE;

                if self.encrypting.get() {
                    // Check the mac
                    let mut mac = self.mac.take().unwrap();
                    mac.update_padded(
                        &crypt_buf[(message_offset + AES128_BLOCK_SIZE)
                            ..(message_offset + message_len + AES128_BLOCK_SIZE)],
                    );

                    buf[0..message_len]
                        .copy_from_slice(&crypt_buf[copy_offset..(copy_offset + message_len)]);

                    let associated_data_bits = ((message_offset - aad_offset) as u64) * 8;
                    let buffer_bits = (message_len as u64) * 8;

                    let mut block = ghash::Block::default();
                    block[..8].copy_from_slice(&associated_data_bits.to_be_bytes());
                    block[8..].copy_from_slice(&buffer_bits.to_be_bytes());
                    mac.update(&block);

                    let mut tag = mac.finalize().into_bytes();

                    for i in 0..AES128_BLOCK_SIZE {
                        tag[i] ^= crypt_buf[tag_offset + i];
                    }

                    buf[(message_offset + message_len)
                        ..(message_offset + message_len + AES128_BLOCK_SIZE)]
                        .copy_from_slice(&tag);
                } else {
                    buf[0..message_len]
                        .copy_from_slice(&crypt_buf[copy_offset..(copy_offset + message_len)]);
                }

                self.aes.disable();
                self.crypt_buf.replace(crypt_buf);
                self.state.set(GCMState::Idle);
                self.gcm_client.map(move |client| {
                    client.crypt_done(buf, Ok(()), true);
                });
            }
        }
    }
}
