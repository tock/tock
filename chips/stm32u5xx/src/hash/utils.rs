// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! HASH utilities for leftovers, HMAC key, adapters and distributed client handling.

use core::cell::Cell;

use kernel::ErrorCode;
use kernel::hil::digest;
use kernel::utilities::cells::MapCell;
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};

use crate::hash::md5::Md5Adapter;
use crate::hash::sha1::Sha1Adapter;
use crate::hash::sha224::Sha224Adapter;
use crate::hash::sha256::Sha256Adapter;

const MAX_HMAC_KEY_LEN: usize = 128;

#[derive(Clone, Copy)]
pub enum Mode {
    MD5,
    SHA1,
    SHA2_224,
    SHA2_256,
}

impl Mode {
    pub fn get_digest_len(&self) -> usize {
        match self {
            Mode::MD5 => 4,
            Mode::SHA1 => 5,
            Mode::SHA2_224 => 7,
            Mode::SHA2_256 => 8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    Add,
    PreRun,
    Run,
    HmacInit,
    HmacPreAuth,
    HmacPostAuth,
    HmacFinalize,
}

#[derive(Clone, Copy)]
pub enum HashAdapter<'a> {
    Md5(&'a Md5Adapter<'a>),
    Sha1(&'a Sha1Adapter<'a>),
    Sha224(&'a Sha224Adapter<'a>),
    Sha256(&'a Sha256Adapter<'a>),
}

#[derive(Clone, Copy)]
pub enum HashClient<'a, const DIGEST_LEN: usize> {
    // Unique clients for every operation
    Split(
        Option<&'a dyn digest::ClientData<DIGEST_LEN>>,
        Option<&'a dyn digest::ClientHash<DIGEST_LEN>>,
        Option<&'a dyn digest::ClientVerify<DIGEST_LEN>>,
    ),
    DataHasher(&'a dyn digest::ClientDataHash<DIGEST_LEN>),
    DataVerifier(&'a dyn digest::ClientDataVerify<DIGEST_LEN>),
    AllInOne(&'a dyn digest::Client<DIGEST_LEN>),
}

// Implements all the callback functions that are used by every client in the HIL
impl<const DIGEST_LEN: usize> HashClient<'_, DIGEST_LEN> {
    pub fn add_data_done(&self, result: Result<(), ErrorCode>, data: SubSlice<'static, u8>) {
        match self {
            Self::Split(client_data, _, _) => {
                client_data.map(|c| c.add_data_done(result, data));
            }
            Self::DataHasher(client) => client.add_data_done(result, data),
            Self::DataVerifier(client) => client.add_data_done(result, data),
            Self::AllInOne(client) => client.add_data_done(result, data),
        }
    }

    pub fn add_mut_data_done(&self, result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>) {
        match self {
            Self::Split(client_data, _, _) => {
                client_data.map(|c| c.add_mut_data_done(result, data));
            }
            Self::DataHasher(client) => client.add_mut_data_done(result, data),
            Self::DataVerifier(client) => client.add_mut_data_done(result, data),
            Self::AllInOne(client) => client.add_mut_data_done(result, data),
        }
    }

    pub fn hash_done(&self, result: Result<(), ErrorCode>, digest: &'static mut [u8; DIGEST_LEN]) {
        match self {
            Self::Split(_, client_hash, _) => {
                client_hash.map(|c| c.hash_done(result, digest));
            }
            Self::DataHasher(client) => client.hash_done(result, digest),
            Self::AllInOne(client) => client.hash_done(result, digest),
            _ => (),
        }
    }

    pub fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        compare: &'static mut [u8; DIGEST_LEN],
    ) {
        match self {
            Self::Split(_, _, client_verify) => {
                client_verify.map(|c| c.verification_done(result, compare));
            }
            Self::DataVerifier(client) => client.verification_done(result, compare),
            Self::AllInOne(client) => client.verification_done(result, compare),
            _ => (),
        }
    }
}

pub struct Leftover {
    buffer: Cell<Option<u32>>,
    index: Cell<usize>,
}

impl Leftover {
    pub fn new() -> Self {
        Leftover {
            buffer: Cell::new(None),
            index: Cell::new(0),
        }
    }

    /// Add a new byte to the leftover buffer.
    pub fn add(&self, byte: u8) {
        if !self.is_full() {
            self.buffer.update(|buf| match buf {
                // Example of the operation
                // 01 -> 01xxxxxx
                // 02 -> 0201xxxx
                // 03 -> 030201xx
                // 04 -> 04030201
                Some(b) => Some(b >> 8 | (byte as u32).rotate_right(8)),
                None => Some((byte as u32).rotate_right(8)),
            });
        }

        self.index.update(|index| (index + 1) % 5);
    }

    /// Empty the buffer
    pub fn empty(&self) {
        self.buffer.take();
    }

    /// Return the contents of the buffer in little endian format
    pub fn to_le(&self) -> u32 {
        match self.buffer.take() {
            Some(b) => {
                let value = b >> (8 * self.bytes_left());
                self.index.update(|idx| idx.saturating_sub(4));
                value
            }
            None => 0,
        }
    }

    /// Returns how many bytes are left to fill the buffer up.
    pub fn bytes_left(&self) -> usize {
        4 - self.index.get()
    }

    /// Returns if the buffer full or not.
    pub fn is_full(&self) -> bool {
        self.index.get() == 4 && self.buffer.get().is_some()
    }

    /// Returns if the buffer empty or not.
    pub fn is_empty(&self) -> bool {
        self.buffer.get().is_none()
    }
}

// HMAC key helping struct
pub struct HmacKey {
    pub key: MapCell<[u8; MAX_HMAC_KEY_LEN]>,
    pub index: Cell<usize>,
    len: Cell<usize>,
}

impl HmacKey {
    pub fn new() -> Self {
        Self {
            key: MapCell::empty(),
            index: Cell::new(0),
            len: Cell::new(0),
        }
    }

    /// Save the key.
    ///
    /// Returns `Ok(())` if save was successful.
    ///
    /// Returns `Error(ErrorCode::SIZE)` if the sent key is bigger than `MAX_HMAC_KEY_LEN`.
    ///
    /// Returns `Error(ErrorCode::FAIL)` if the key buffer was not allocated.
    pub fn set(&self, key: &[u8]) -> Result<(), kernel::ErrorCode> {
        if self.key.is_none() {
            self.key.put([0u8; MAX_HMAC_KEY_LEN]);
        }
        match self.key.map(|buf| {
            if buf.len() >= key.len() {
                buf[..key.len()].copy_from_slice(key);
                self.len.set(key.len());
                Ok(())
            } else {
                Err(ErrorCode::SIZE)
            }
        }) {
            Some(r) => r,
            None => Err(ErrorCode::FAIL),
        }
    }

    /// Checks if the key is stored by the peripheral struct.
    pub fn is_stored(&self) -> bool {
        self.key.is_some()
    }

    /// Checks if the key is loaded into hardware.
    pub fn is_loaded(&self) -> bool {
        self.index.get() == self.len.get()
    }

    /// Returns the number of bytes that are left to load the key completely.
    pub fn left_to_load(&self) -> usize {
        self.len.get().saturating_sub(self.index.get())
    }

    /// Resets the index of the key buffer and makes it available for loading again.
    pub fn reset_index(&self) {
        if self.key.is_some() {
            self.index.take();
        }
    }

    /// Empties the buffer, its length and index.
    pub fn clear(&self) {
        self.key.take();
        self.len.take();
        self.index.take();
    }

    /// Returns length of the HMAC key.
    pub fn len(&self) -> usize {
        if let Some(key) = self.key.get() {
            key.len()
        } else {
            0
        }
    }
}
